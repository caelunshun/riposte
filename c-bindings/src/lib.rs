//! C bindings to Riposte's networking internals, including QUIC
//! support. All IO is asynchronous and poll-based. Messages
//! are sent length-prefixed using a consistent protocol.
//!
//! # Request model
//! Asynchronous operations are modeled as _requests_. Each request
//! represents some ongoing IO operation, like sending a message or
//! connecting to an endpoint. Every request is also associated
//! with a _callback_ that is invoked once the request completes.
//!
//! Call [`networkctx_wait`] to wait on the next completed request and
//! invoke its callback.

use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::slice;
use std::sync::Arc;

use anyhow::Context;
use bytes::{Bytes, BytesMut};
use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use quinn::{Connection, IncomingUniStreams, RecvStream};
use riposte_backend_api::prost::Message;
use riposte_backend_api::uuid::Uuid;
use riposte_backend_api::{
    codec, riposte_backend_client::RiposteBackendClient, tonic::transport::Channel, BACKEND_URL,
};
use riposte_backend_api::{
    open_stream, quic_addr, CreateGameRequest, FramedRead, LengthDelimitedCodec, OpenStream,
    ProxiedStream,
};
use rustls::ServerCertVerified;
use slotmap::SlotMap;
use tokio::sync::Mutex;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    runtime::{self, Runtime},
    task,
};
use zeno::{PathBuilder, Placement};

slotmap::new_key_type! {
    pub struct RequestId;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum RipError {
    None,
    Io,
}

pub enum RipResultData {
    None,
    Bytes(BytesMut),
    Connection(*mut RipConnectionHandle, Uuid),
}

unsafe impl Send for RipResultData {}
unsafe impl Sync for RipResultData {}

pub struct RipResult {
    pub success: bool,
    pub error: RipError,
    pub data: RipResultData,
}

impl RipResult {
    pub fn success(data: RipResultData) -> Self {
        Self {
            success: true,
            error: RipError::None,
            data,
        }
    }

    pub fn error(error: RipError) -> Self {
        Self {
            success: false,
            error,
            data: RipResultData::None,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rip_result_is_success(res: &RipResult) -> bool {
    res.success
}

#[no_mangle]
pub unsafe extern "C" fn rip_result_get_error(res: &RipResult) -> RipError {
    res.error
}

#[repr(C)]
pub struct RipBytes {
    pub len: usize,
    pub ptr: *const u8,
}

#[no_mangle]
pub unsafe extern "C" fn rip_result_get_bytes(res: &RipResult) -> RipBytes {
    if let RipResultData::Bytes(bytes) = &res.data {
        RipBytes {
            len: bytes.len(),
            ptr: bytes.as_ptr(),
        }
    } else {
        panic!("RipResult did not contain Bytes")
    }
}

#[no_mangle]
pub unsafe extern "C" fn rip_result_get_connection(res: &RipResult) -> *mut RipConnectionHandle {
    if let RipResultData::Connection(conn, _) = &res.data {
        *conn
    } else {
        panic!("RipResult did not contain a connection")
    }
}

#[no_mangle]
pub unsafe extern "C" fn rip_result_get_connection_uuid(res: &RipResult) -> *const c_char {
    if let RipResultData::Connection(_, uuid) = &res.data {
        let s = CString::new(uuid.to_hyphenated().to_string()).unwrap();
        let s: Box<[u8]> = s.as_bytes_with_nul().to_vec().into_boxed_slice();
        Box::leak(s).as_ptr() as *const c_char
    } else {
        panic!("RipResult did not contain a connection")
    }
}

/// Asserts that any value in the `data` field of `RipResult` is always `Send`.
unsafe impl Send for RipResult {}

/// Parameter 1: userdata passed into request
/// Parameter 2: result of request
pub type Callback = extern "C" fn(*mut c_void, &RipResult);

struct CompletedRequest {
    id: RequestId,
    result: RipResult,
}

// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
struct SkipCertificationVerification;
impl rustls::ServerCertVerifier for SkipCertificationVerification {
    fn verify_server_cert(
        &self,
        _: &rustls::RootCertStore,
        _: &[rustls::Certificate],
        _: webpki::DNSNameRef,
        _: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}

/// A networking context. Internally
/// stores the Tokio runtime instance.
///
/// Maintains a list of callbacks, one for each
/// request performed.
pub struct RipNetworkingContext {
    runtime: Runtime,
    callbacks: SlotMap<RequestId, (Callback, *mut c_void)>,
    completed_requests: Receiver<CompletedRequest>,
    completed_requests_sender: Sender<CompletedRequest>,

    client: RiposteBackendClient<Channel>,

    endpoint: quinn::Endpoint,
}

impl RipNetworkingContext {
    pub fn insert_callback(&mut self, callback: Callback, userdata: *mut c_void) -> RequestId {
        self.callbacks.insert((callback, userdata))
    }
}

/// Creates a new networking context.
#[no_mangle]
pub unsafe extern "C" fn networkctx_create() -> *mut RipNetworkingContext {
    let (completed_requests_sender, completed_requests) = flume::bounded(16);

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create runtime");

    let _guard = runtime.enter();

    let client = runtime
        .block_on(async move { RiposteBackendClient::connect(BACKEND_URL).await })
        .expect("failed to connect to backend");

    let mut config = quinn::ClientConfigBuilder::default().build();
    let tls_cfg: &mut rustls::ClientConfig = Arc::get_mut(&mut config.crypto).unwrap();
    tls_cfg
        .dangerous()
        .set_certificate_verifier(Arc::new(SkipCertificationVerification));

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.default_client_config(config);
    let (endpoint, _) = endpoint
        .bind(&"0.0.0.0:0".parse().unwrap())
        .expect("failed to bind to QUIC");

    let ctx = RipNetworkingContext {
        runtime,
        callbacks: SlotMap::default(),
        completed_requests,
        completed_requests_sender,
        client,
        endpoint,
    };
    Box::leak(Box::new(ctx))
}

/// Frees a networking context.
#[no_mangle]
pub unsafe extern "C" fn networkctx_free(ctx: *mut RipNetworkingContext) {
    Box::from_raw(ctx);
}

/// Waits for one completed request and invokes its
/// callback.
///
/// Should be called in a loop to process more than one request.
#[no_mangle]
pub unsafe extern "C" fn networkctx_wait(ctx: *mut RipNetworkingContext) {
    let ctx = unpointer(ctx);

    let request = ctx
        .completed_requests
        .recv()
        .expect("channel disconnected?");

    if let Some((callback, userdata)) = ctx.callbacks.remove(request.id) {
        callback(userdata, &request.result);
    }
}

#[no_mangle]
pub unsafe extern "C" fn networkctx_create_game(
    ctx: &mut RipNetworkingContext,
    host_auth_token: *const u8,
    host_auth_token_len: usize,
) -> *mut RipHubServerConnection {
    let auth_token = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
        host_auth_token,
        host_auth_token_len,
    ));
    let auth_token = base64::decode(&auth_token).unwrap();

    dbg!();
    let mut client = ctx.client.clone();
    let session_id = ctx
        .runtime
        .block_on(async move { client.create_game(CreateGameRequest { auth_token }).await })
        .expect("failed to create game")
        .into_inner()
        .session_id;

    dbg!();

    let endpoint = &ctx.endpoint;
    let new_conn = ctx.runtime.block_on(async move {
        endpoint
            .connect(&quic_addr(), "localhost")
            .expect("failed to connect")
            .await
            .expect("failed to connect")
    });

    dbg!();

    let conn = &new_conn.connection;
    ctx.runtime.block_on(async move {
        let mut stream = conn.open_uni().await.unwrap();
        stream.write_all(&session_id).await.unwrap();
    });

    dbg!();

    let _guard = ctx.runtime.enter();

    Box::leak(Box::new(RipHubServerConnection::new(ctx, new_conn))) as *mut _
}

struct GetNewConnectionRequest {
    request_id: RequestId,
}

type Connections =
    Arc<Mutex<HashMap<Uuid, (Receiver<SendDataRequest>, Receiver<RecvDataRequest>)>>>;

pub struct RipHubServerConnection {
    new_connections_req_tx: Sender<GetNewConnectionRequest>,
}

impl RipHubServerConnection {
    pub fn new(ctx: &RipNetworkingContext, new_conn: quinn::NewConnection) -> Self {
        let (new_connections_req_tx, new_connections_req_rx) = flume::unbounded();

        let completed_requests = ctx.completed_requests_sender.clone();

        let connections = Connections::default();

        let incoming_uni = new_conn.uni_streams;
        let conn = new_conn.connection;
        task::spawn(async move {
            if let Err(e) = handle_backend_streams(
                incoming_uni,
                connections,
                completed_requests,
                new_connections_req_rx,
                conn,
            )
            .await
            {
                eprintln!("Connection to backend lost: {:?}", e);
            }
        });

        Self {
            new_connections_req_tx,
        }
    }
}

async fn handle_backend_streams(
    mut incoming: IncomingUniStreams,
    connections: Connections,
    completed_requests: Sender<CompletedRequest>,
    new_connections_req_rx: Receiver<GetNewConnectionRequest>,
    conn: Connection,
) -> anyhow::Result<()> {
    loop {
        let backend_stream = incoming.next().await.context("no stream")??;
        eprintln!("Remote opened a stream");
        let backend_reader = codec().new_read(backend_stream);

        let connections1 = connections.clone();
        let completed_requests1 = completed_requests.clone();
        let new_connections_req_rx1 = new_connections_req_rx.clone();
        let conn1 = conn.clone();
        task::spawn(async move {
            if let Err(e) = handle_backend_stream(
                backend_reader,
                connections1,
                completed_requests1,
                new_connections_req_rx1,
                conn1,
            )
            .await
            {
                eprintln!("Failed to handle backend stream: {:?}", e);
            }
        });
    }
}

async fn handle_backend_stream(
    mut reader: FramedRead<RecvStream, LengthDelimitedCodec>,
    connections: Connections,
    completed_requests: Sender<CompletedRequest>,
    new_connections_req_rx: Receiver<GetNewConnectionRequest>,
    quic_conn: Connection,
) -> anyhow::Result<()> {
    let data = reader.next().await.unwrap().unwrap();
    let packet = OpenStream::decode(data.as_ref()).unwrap();
    eprintln!("Stream type: {:?}", packet);
    match packet.inner.unwrap() {
        open_stream::Inner::NewClient(new_client) => {
            if let Ok(GetNewConnectionRequest { request_id }) =
                new_connections_req_rx.recv_async().await
            {
                let (sending_data, sending_data_recv) = flume::unbounded();
                let (receiving_data, receiving_data_recv) = flume::unbounded();

                let conn = RipConnectionHandle {
                    sending_data,
                    receiving_data,
                    current_recv_id: Default::default(),
                };

                connections.lock().await.insert(
                    new_client.connection_id.clone().unwrap_or_default().into(),
                    (sending_data_recv.clone(), receiving_data_recv.clone()),
                );

                // Spawn task to send data
                let send_stream = quic_conn.open_uni().await?;
                let mut writer = codec().new_write(send_stream);

                writer
                    .send(
                        OpenStream {
                            inner: Some(open_stream::Inner::ProxiedStream(ProxiedStream {
                                connection_id: new_client.connection_id.clone(),
                            })),
                        }
                        .encode_to_vec()
                        .into(),
                    )
                    .await?;

                let completed_requests1 = completed_requests.clone();
                task::spawn(async move {
                    while let Ok(req) = sending_data_recv.recv_async().await {
                        writer.send(req.data).await?;
                        completed_requests1.send(CompletedRequest {
                            id: req.request_id,
                            result: RipResult::success(RipResultData::None),
                        })?;
                    }
                    Result::<(), anyhow::Error>::Ok(())
                });

                let conn = Box::leak(Box::new(conn)) as *mut _;

                completed_requests
                    .send(CompletedRequest {
                        id: request_id,
                        result: RipResult::success(RipResultData::Connection(
                            conn,
                            new_client.player_uuid.unwrap_or_default().into(),
                        )),
                    })
                    .ok();
            }
        }
        open_stream::Inner::ProxiedStream(proxied_stream) => {
            let (_, receiving_data_rx) = connections.lock().await[&proxied_stream
                .connection_id
                .clone()
                .unwrap_or_default()
                .into()]
                .clone();

            while let Ok(RecvDataRequest { request_id }) = receiving_data_rx.recv_async().await {
                let msg = reader.next().await.context("end of stream")??;
                completed_requests.send(CompletedRequest {
                    id: request_id,
                    result: RipResult::success(RipResultData::Bytes(msg)),
                })?;
            }
        }
        open_stream::Inner::ClientDisconnected(_) => {} // TODO
    }

    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn hubconn_get_new_connection(
    ctx: &mut RipNetworkingContext,
    conn: &mut RipHubServerConnection,
    callback: Callback,
    userdata: *mut c_void,
) {
    let request_id = ctx.insert_callback(callback, userdata);
    conn.new_connections_req_tx
        .send(GetNewConnectionRequest { request_id })
        .unwrap();
}

struct SendDataRequest {
    data: Bytes,
    request_id: RequestId,
}

struct RecvDataRequest {
    request_id: RequestId,
}

/// Handle to a connected peer.
///
/// The connection runs over any IO system, such as
/// a QUIC endpoint or stdout/stdin. Messages
/// are length-prefixed.
pub struct RipConnectionHandle {
    sending_data: Sender<SendDataRequest>,
    receiving_data: Sender<RecvDataRequest>,
    current_recv_id: RequestId,
}

/// Creates a new connection handle operating on stdout/stdin.
///
/// Used in singleplayer mode when the server runs as a child process
/// of the client.
#[no_mangle]
pub unsafe extern "C" fn networkctx_connect_stdio(
    ctx: &mut RipNetworkingContext,
) -> *mut RipConnectionHandle {
    let _runtime_guard = ctx.runtime.enter();

    let stdin = io::stdin();
    let stdout = io::stdout();

    let (sending_data, sending_data_recv) = flume::unbounded();
    let (receiving_data, receiving_data_recv) = flume::unbounded();

    run_connection(
        stdin,
        stdout,
        sending_data_recv,
        receiving_data_recv,
        ctx.completed_requests_sender.clone(),
    );

    Box::leak(Box::new(RipConnectionHandle {
        sending_data,
        receiving_data,
        current_recv_id: RequestId::default(),
    }))
}

/// Creates a request that sends data to the given connection.
///
/// `callback` is invoked after the data is sent. The `data` field
/// is always set to `None`.
///
/// `data` is copied; we don't take ownership.
#[no_mangle]
pub unsafe extern "C" fn networkctx_conn_send_data(
    ctx: &mut RipNetworkingContext,
    conn: &RipConnectionHandle,
    data: RipBytes,
    callback: Callback,
    userdata: *mut c_void,
) {
    let request_id = ctx.insert_callback(callback, userdata);
    conn.sending_data
        .send(SendDataRequest {
            data: slice::from_raw_parts(data.ptr, data.len).to_vec().into(),
            request_id,
        })
        .ok();
}

/// Creates a request that receives data from the given connection.
///
/// `callback` is invoked after the data is received. The `data` field
/// contains a `bytes` variant if successful.
#[no_mangle]
pub unsafe extern "C" fn networkctx_conn_recv_data(
    ctx: &mut RipNetworkingContext,
    conn: &mut RipConnectionHandle,
    callback: Callback,
    userdata: *mut c_void,
) {
    // If an existing request for data already exists, override it.
    let request_id = if ctx.callbacks.contains_key(conn.current_recv_id) {
        *(&mut ctx.callbacks[conn.current_recv_id]) = (callback, userdata);
        conn.current_recv_id
    } else {
        let request_id = ctx.insert_callback(callback, userdata);
        conn.receiving_data
            .send(RecvDataRequest { request_id })
            .ok();
        request_id
    };
    conn.current_recv_id = request_id;
}

/// Frees a connection, disconnecting it.
#[no_mangle]
pub unsafe extern "C" fn networkctx_conn_free(
    _ctx: &mut RipNetworkingContext,
    conn: *mut RipConnectionHandle,
) {
    // Dropping the connection causes its channels to disconnect,
    // which stops the two connection tasks.
    drop(Box::from_raw(conn));
}

fn run_connection(
    reader: impl AsyncRead + Unpin + Send + 'static,
    writer: impl AsyncWrite + Unpin + Send + 'static,
    sending_data: Receiver<SendDataRequest>,
    receiving_data: Receiver<RecvDataRequest>,
    completed_requests: Sender<CompletedRequest>,
) {
    let mut reader = codec().new_read(reader);
    let mut writer = codec().new_write(writer);

    // Sending data task
    let completed_requests2 = completed_requests.clone();
    task::spawn(async move {
        while let Ok(req) = sending_data.recv_async().await {
            let result = match writer.send(req.data).await {
                Ok(_) => RipResult::success(RipResultData::None),
                Err(_) => RipResult::error(RipError::Io),
            };
            completed_requests2
                .send_async(CompletedRequest {
                    id: req.request_id,
                    result,
                })
                .await
                .ok();
        }
    });

    // Receiving data task
    task::spawn(async move {
        // Wait for receive requests before actually receiving data.
        while let Ok(req) = receiving_data.recv_async().await {
            let result = match reader.next().await {
                Some(Ok(res)) => RipResult::success(RipResultData::Bytes(res)),
                _ => RipResult::error(RipError::Io),
            };
            completed_requests
                .send_async(CompletedRequest {
                    id: req.request_id,
                    result,
                })
                .await
                .ok();
        }
    });
}

pub struct RipRasterizedMask {
    data: Vec<u8>,
    placement: Placement,
}

#[no_mangle]
pub unsafe extern "C" fn zeno_mask_get_value(mask: &RipRasterizedMask, x: u32, y: u32) -> u8 {
    mask.data[(x + y * mask.placement.width) as usize]
}

#[no_mangle]
pub unsafe extern "C" fn zeno_mask_get_width(mask: &RipRasterizedMask) -> u32 {
    mask.placement.width
}

#[no_mangle]
pub unsafe extern "C" fn zeno_mask_get_height(mask: &RipRasterizedMask) -> u32 {
    mask.placement.height
}

#[no_mangle]
pub unsafe extern "C" fn zeno_mask_free(mask: *mut RipRasterizedMask) {
    drop(Box::from_raw(mask));
}

/// Rasterizes the given line mesh into an alpha grid.
#[no_mangle]
pub unsafe extern "C" fn zeno_rasterize_lines(
    coordinates: *const f32,
    num_points: usize,
) -> *mut RipRasterizedMask {
    let mut path: Vec<zeno::Command> = Vec::new();
    let coordinates = slice::from_raw_parts(coordinates, num_points * 2);

    for (i, point) in coordinates.chunks_exact(2).enumerate() {
        let x = point[0];
        let y = point[1];

        if i == 0 {
            path.move_to([x, y]);
        } else {
            path.line_to([x, y]);
        }
    }

    <Vec<zeno::Command> as PathBuilder>::close(&mut path);

    let (data, placement) = zeno::Mask::new(&path).render();
    Box::leak(Box::new(RipRasterizedMask { data, placement })) as *mut _
}

unsafe fn unpointer<T>(ptr: *mut T) -> &'static mut T {
    &mut *ptr
}
