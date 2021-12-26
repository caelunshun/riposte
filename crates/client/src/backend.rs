use anyhow::Context as _;
use riposte_backend_api::{
    grpc_server_addr,
    riposte_backend_client::RiposteBackendClient,
    tonic::{
        transport::{Channel, ClientTlsConfig},
        Response, Status,
    },
    Authenticated, GameList, GameListRequest, LogInRequest, RegisterRequest, UserInfo,
};
use tokio::runtime;
use uuid::Uuid;

use crate::context::FutureHandle;

pub type BackendResponse<T> = FutureHandle<Result<Response<T>, Status>>;

/// Maintains a connection to the gRPC backend service,
/// which handles user authentication and multiplayer server lists.
pub struct BackendService {
    client: RiposteBackendClient<Channel>,
    runtime: runtime::Handle,
}

impl BackendService {
    pub async fn new(runtime: runtime::Handle) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(format!("http://{}", grpc_server_addr()))?
            .tls_config(ClientTlsConfig::new().domain_name("riposte.tk"))?
            .connect()
            .await
            .context("failed to connect to Riposte backend service. Check your Internet.")?;
        let client = RiposteBackendClient::new(channel);

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

    pub fn fetch_user_data(&self, user_id: Uuid) -> BackendResponse<UserInfo> {
        let mut client = self.client.clone();
        FutureHandle::spawn(&self.runtime, async move {
            log::info!("Fetching user info for {:?}", user_id);
            let res = client
                .fetch_user_info(riposte_backend_api::Uuid::from(user_id))
                .await;
            if let Err(e) = &res {
                log::error!("Failed to fetch user info for {:?}: {}", user_id, e);
            }
            res
        })
    }

    pub fn list_games(&self) -> BackendResponse<GameList> {
        let mut client = self.client.clone();
        FutureHandle::spawn(&self.runtime, async move {
            let res = client.request_game_list(GameListRequest {}).await;
            if let Err(e) = &res {
                log::error!("Failed to fetch game list: {}", e);
            }
            res
        })
    }
}
