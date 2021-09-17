use std::{path::PathBuf, process::Stdio};

use anyhow::Context as _;
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures::{stream::StreamExt, SinkExt};
use riposte_backend_api::{codec, join_game_response, quic_addr, JoinGameRequest, Uuid};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    task,
};

use crate::{
    context::{Context, FutureHandle},
    options::Account,
};

fn server_path() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .unwrap()
        .join("riposte-server"))
}

/// A bridge abstracting over a connection to the game server.
#[derive(Clone)]
pub struct ServerBridge {
    sending: Sender<Bytes>,
    receiving: Receiver<Bytes>,
}

impl ServerBridge {
    /// Launches a singleplayer game server process.
    ///
    /// Returns a connection to it.
    pub fn create_singleplayer(host_account: &Account) -> anyhow::Result<Self> {
        let mut server_process = Command::new(server_path()?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env(
                "RIPOSTE_HOST_UUID",
                host_account.uuid().to_hyphenated().to_string(),
            )
            .env(
                "RIPOSTE_HOST_AUTH_TOKEN",
                base64::encode(host_account.auth_token()),
            )
            .kill_on_drop(true)
            .spawn()
            .context("failed to launch server process")?;

        let stdin = server_process.stdin.take().unwrap();
        let stdout = server_process.stdout.take().unwrap();
        let stderr = server_process.stderr.take().unwrap();

        task::spawn(async move {
            let exit_code = server_process
                .wait()
                .await
                .expect("failed to await process");
            if !exit_code.success() {
                log::error!("Server process exited with code {}", exit_code);
            }
        });

        let (sending_tx, sending_rx) = flume::unbounded();
        let (receiving_tx, receiving_rx) = flume::unbounded();

        let mut reader = codec().new_read(stdout);
        let mut writer = codec().new_write(stdin);

        task::spawn(async move {
            while let Some(Ok(msg)) = reader.next().await {
                // Uncomment to add artificial ping for testing.
                // tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                if receiving_tx.send_async(msg.freeze()).await.is_err() {
                    break;
                }
            }
        });

        task::spawn(async move {
            while let Ok(msg) = sending_rx.recv_async().await {
                if writer.send(msg).await.is_err() {
                    break;
                }
            }
        });

        task::spawn(async move {
            let mut stderr_lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                log::info!("{}", line);
            }
        });

        Ok(Self {
            sending: sending_tx,
            receiving: receiving_rx,
        })
    }

    /// Creates a new multiplayer server connection by
    /// connecting the Riposte backend service.
    pub fn new_multiplayer(cx: &Context, game_id: Uuid) -> FutureHandle<anyhow::Result<Self>> {
        let mut client = cx.backend().client().clone();
        let quic_endpoint = cx.backend().quic_endpoint().clone();
        let auth_token = cx.options().account().auth_token().to_vec();
        cx.spawn_future(async move {
            let response = client
                .join_game(JoinGameRequest {
                    game_id: Some(game_id),
                    auth_token,
                })
                .await?
                .into_inner();
            log::info!("Join game response: {:?}", response);

            let session_id = match response.result.context("missing result")? {
                join_game_response::Result::ErrorMessage(e) => anyhow::bail!("{}", e),
                join_game_response::Result::SessionId(session_id) => session_id,
            };

            let new_conn = quic_endpoint.connect(&quic_addr(), "localhost")?.await?;
            log::info!("Connected to backend QUIC endpoint");
            let mut stream = new_conn.connection.open_uni().await?;
            stream.write_all(&session_id).await?;
            log::info!("Sent session ID to  backend");

            let send_stream = new_conn.connection.open_uni().await?;

            log::info!("Opened initial send stream");

            let (sending_tx, sending_rx) = flume::unbounded();
            let (receiving_tx, receiving_rx) = flume::unbounded();

            let mut incoming = new_conn.uni_streams;

            task::spawn(async move {
                loop {
                    let recv_stream = incoming.next().await.context("no stream")??;
                    let mut reader = codec().new_read(recv_stream);

                    let receiving_tx = receiving_tx.clone();
                    task::spawn(async move {
                        while let Some(Ok(bytes)) = reader.next().await {
                            receiving_tx.send_async(bytes.freeze()).await?;
                        }
                        Result::<(), anyhow::Error>::Ok(())
                    });
                }
                #[allow(unreachable_code)] // needed to indicate task return type
                Result::<(), anyhow::Error>::Ok(())
            });
            task::spawn(async move {
                let mut writer = codec().new_write(send_stream);
                while let Ok(data) = sending_rx.recv_async().await {
                    if let Err(e) = writer.send(data).await {
                        log::error!("Failed to send data: {:?}", e);
                    }
                }
            });

            log::info!("Server bridge connected to backend");

            Ok(Self {
                sending: sending_tx,
                receiving: receiving_rx,
            })
        })
    }

    /// Polls for the next received message.
    pub fn poll_for_message(&self) -> Option<Bytes> {
        self.receiving.try_recv().ok()
    }

    /// Sends a message.
    pub fn send_message(&self, msg: Bytes) {
        self.sending.send(msg).ok();
    }

    /// Returns whether the server is still connected.
    pub fn is_connected(&self) -> bool {
        !self.sending.is_disconnected()
    }
}
