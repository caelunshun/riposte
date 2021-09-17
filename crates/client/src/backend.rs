use anyhow::Context as _;
use riposte_backend_api::{
    riposte_backend_client::RiposteBackendClient,
    tonic::{transport::Channel, Response, Status},
    Authenticated, GameList, GameListRequest, LogInRequest, RegisterRequest, UserInfo, BACKEND_URL,
};
use tokio::runtime;
use uuid::Uuid;

use std::sync::Arc;

use crate::context::FutureHandle;

pub type BackendResponse<T> = FutureHandle<Result<Response<T>, Status>>;

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
        Ok(rustls::ServerCertVerified::assertion())
    }
}

/// Maintains a connection to the gRPC backend service,
/// which handles user authentication and multiplayer server lists.
pub struct BackendService {
    client: RiposteBackendClient<Channel>,
    runtime: runtime::Handle,
    endpoint: quinn::Endpoint,
}

impl BackendService {
    pub async fn new(runtime: runtime::Handle) -> anyhow::Result<Self> {
        let client = RiposteBackendClient::connect(BACKEND_URL)
            .await
            .context("failed to connect to Riposte backend service. Check your Internet.")?;

        log::info!("Connected to Riposte backend service.");

        let mut endpoint_config = quinn::ClientConfigBuilder::default().build();
        let tls_cfg: &mut rustls::ClientConfig = Arc::get_mut(&mut endpoint_config.crypto).unwrap();
        tls_cfg
            .dangerous()
            .set_certificate_verifier(Arc::new(SkipCertificationVerification));

        let mut endpoint = quinn::Endpoint::builder();
        endpoint.default_client_config(endpoint_config);
        let (endpoint, _) = endpoint
            .bind(&"0.0.0.0:0".parse().unwrap())
            .context("failed to bind to QUIC socket")?;

        Ok(Self {
            client,
            runtime,
            endpoint,
        })
    }

    pub fn client(&self) -> &RiposteBackendClient<Channel> {
        &self.client
    }

    pub fn quic_endpoint(&self) -> &quinn::Endpoint {
        &self.endpoint
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
