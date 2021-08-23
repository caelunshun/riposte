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

use std::ffi::c_void;
use std::slice;

use bytes::{Bytes, BytesMut};
use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use riposte_backend_api::codec;
use slotmap::SlotMap;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    runtime::{self, Runtime},
    task,
};

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
}

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

/// Asserts that any value in the `data` field of `RipResult` is always `Send`.
unsafe impl Send for RipResult {}

/// Parameter 1: userdata passed into request
/// Parameter 2: result of request
pub type Callback = extern "C" fn(*mut c_void, &RipResult);

struct CompletedRequest {
    id: RequestId,
    result: RipResult,
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

    let ctx = RipNetworkingContext {
        runtime: runtime::Builder::new_multi_thread()
            .build()
            .expect("failed to create runtime"),
        callbacks: SlotMap::default(),
        completed_requests,
        completed_requests_sender,
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
            data: slice::from_raw_parts(data.ptr, data.len).into(),
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
    conn: &RipConnectionHandle,
    callback: Callback,
    userdata: *mut c_void,
) {
    let request_id = ctx.insert_callback(callback, userdata);
    conn.receiving_data
        .send(RecvDataRequest { request_id })
        .ok();
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

unsafe fn unpointer<T>(ptr: *mut T) -> &'static mut T {
    &mut *ptr
}
