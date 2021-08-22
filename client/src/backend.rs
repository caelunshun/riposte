use anyhow::Context as _;
use riposte_backend_api::{
    riposte_backend_client::RiposteBackendClient,
    tonic::{transport::Channel, Response, Status},
    Authenticated, LogInRequest, RegisterRequest,
};
use tokio::runtime;

use crate::context::FutureHandle;

const BACKEND_URL: &str = "http://127.0.0.1:80";

pub type BackendResponse<T> = FutureHandle<Result<Response<T>, Status>>;

/// Maintains a connection to the gRPC backend service,
/// which handles user authentication and multiplayer server lists.
pub struct BackendService {
    client: RiposteBackendClient<Channel>,
    runtime: runtime::Handle,
}

impl BackendService {
    pub async fn new(runtime: runtime::Handle) -> anyhow::Result<Self> {
        let client = RiposteBackendClient::connect(BACKEND_URL)
            .await
            .context("failed to connect to Riposte backend service. Check your Internet.")?;

        log::info!("Connected to Riposte backend service.");

        Ok(Self { client, runtime })
    }

    pub fn client(&self) -> &RiposteBackendClient<Channel> {
        &self.client
    }

    pub fn log_in(&self, request: LogInRequest) -> BackendResponse<Authenticated> {
        let mut client = self.client.clone();
        FutureHandle::spawn(&self.runtime, async move { client.log_in(request).await })
    }

    pub fn register_account(&self, request: RegisterRequest) -> BackendResponse<Authenticated> {
        let mut client = self.client.clone();
        FutureHandle::spawn(&self.runtime, async move {
            client.register_account(request).await
        })
    }
}
