use std::fmt::Display;
use std::sync::Arc;

use riposte_backend_api::riposte_backend_server::RiposteBackendServer;
use riposte_backend_api::{
    join_game_response, Authenticated, CreateGameRequest, CreateGameResponse, DeleteGameRequest,
    GameList, GameListRequest, JoinGameRequest, JoinGameResponse, LogInRequest,
    UpdateGameSettingsRequest, UserInfo, Uuid,
};
use riposte_backend_api::{riposte_backend_server::RiposteBackend, RegisterRequest};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tower_http::trace::TraceLayer;
use tracing::{Instrument, Level};

use crate::hub::Hub;
use crate::models::{User, UserAccessToken};
use crate::repository::Repository;

pub async fn run_grpc_server(repo: Arc<dyn Repository>, hub: Arc<Hub>) -> anyhow::Result<()> {
    let layer = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc())
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(RiposteBackendServer::new(RiposteBackendImpl { repo, hub }))
        .serve("0.0.0.0:80".parse()?)
        .instrument(tracing::span!(Level::INFO, "gRPC Service"))
        .await?;

    Ok(())
}

fn internal(e: impl Display) -> tonic::Status {
    Status::internal(e.to_string())
}

pub struct RiposteBackendImpl {
    repo: Arc<dyn Repository>,
    hub: Arc<Hub>,
}

impl RiposteBackendImpl {
    async fn authenticate(&self, auth_token: &[u8]) -> Result<User, Status> {
        let token = self
            .repo
            .get_user_token_by_token(auth_token)
            .await
            .map_err(internal)?;

        match token {
            Some(tok) => Ok(self
                .repo
                .get_user_by_id(tok.user_id())
                .await
                .map_err(internal)?
                .unwrap()),
            None => Err(Status::unauthenticated("invalid auth token")),
        }
    }
}

#[tonic::async_trait]
impl RiposteBackend for RiposteBackendImpl {
    async fn register_account(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<Authenticated>, Status> {
        let payload = request.into_inner();
        let user = User::create(payload.username, payload.email, payload.password)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // Ensure no user exists with the same username.
        if self
            .repo
            .get_user_by_username(user.username())
            .await
            .map_err(internal)?
            .is_some()
        {
            return Err(Status::already_exists("username is taken"));
        }

        self.repo.insert_user(&user).await.map_err(internal)?;

        let token = UserAccessToken::generate_for_user(user.id());
        self.repo
            .insert_user_token(&token)
            .await
            .map_err(internal)?;

        Ok(Response::new(Authenticated {
            username: user.username().to_owned(),
            uuid: Some(user.id().into()),
            auth_token: token.token_bytes().into(),
        }))
    }

    async fn log_in(
        &self,
        request: Request<LogInRequest>,
    ) -> Result<Response<Authenticated>, Status> {
        let user = self
            .repo
            .get_user_by_username(&request.get_ref().username)
            .await
            .map_err(internal)?;

        let failure = Err(Status::unauthenticated("wrong username or password"));
        match user {
            Some(u) => match u.check_password(&request.get_ref().password) {
                Ok(_) => {
                    let token = UserAccessToken::generate_for_user(u.id());
                    self.repo
                        .insert_user_token(&token)
                        .await
                        .map_err(internal)?;
                    Ok(Response::new(Authenticated {
                        username: u.username().to_owned(),
                        uuid: Some(u.id().into()),
                        auth_token: token.token_bytes().into(),
                    }))
                }
                Err(_) => failure,
            },
            None => failure,
        }
    }

    async fn fetch_user_info(&self, request: Request<Uuid>) -> Result<Response<UserInfo>, Status> {
        let user = self
            .repo
            .get_user_by_id(request.into_inner().into())
            .await
            .map_err(internal)?;
        match user {
            Some(u) => Ok(Response::new(UserInfo {
                username: u.username().to_owned(),
            })),
            None => Err(Status::not_found("user not found")),
        }
    }

    async fn create_game(
        &self,
        request: Request<CreateGameRequest>,
    ) -> Result<Response<CreateGameResponse>, Status> {
        let user = self.authenticate(&request.get_ref().auth_token).await?;

        let session_id = self.hub.create_game(user.id()).await;
        Ok(Response::new(CreateGameResponse {
            session_id: session_id.to_vec(),
        }))
    }

    async fn delete_game(
        &self,
        _request: Request<DeleteGameRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("endpoint unimplemented"))
    }

    async fn update_game_settings(
        &self,
        _request: Request<UpdateGameSettingsRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("endpoint unimplemented"))
    }

    async fn join_game(
        &self,
        request: Request<JoinGameRequest>,
    ) -> Result<Response<JoinGameResponse>, Status> {
        let user = self.authenticate(&request.get_ref().auth_token).await?;
        let session_id = self
            .hub
            .join_game(
                request.get_ref().game_id.clone().unwrap_or_default().into(),
                user.id(),
            )
            .await;
        Ok(Response::new(JoinGameResponse {
            result: Some(join_game_response::Result::SessionId(session_id.to_vec())),
        }))
    }

    async fn request_game_list(
        &self,
        _request: Request<GameListRequest>,
    ) -> Result<Response<GameList>, Status> {
        Ok(Response::new(GameList {
            games: self.hub.games(&*self.repo).await.map_err(internal)?,
        }))
    }
}
