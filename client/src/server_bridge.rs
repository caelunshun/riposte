use std::process::Stdio;

use anyhow::Context as _;
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures::{stream::StreamExt, SinkExt};
use riposte_backend_api::codec;
use tokio::{process::Command, task};

use crate::options::Account;

const SERVER_PATH: &str =
    "/Users/caelum/CLionProjects/riposte/cmake-build-debug/bin/riposte";

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
        let mut server_process = Command::new(SERVER_PATH)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .env("RIPOSTE_HOST_UUID", host_account.uuid().to_hyphenated().to_string())
            .kill_on_drop(true)
            .spawn()
            .context("failed to launch server process")?;

        let stdin = server_process.stdin.take().unwrap();
        let stdout = server_process.stdout.take().unwrap();

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

        Ok(Self {
            sending: sending_tx,
            receiving: receiving_rx,
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
