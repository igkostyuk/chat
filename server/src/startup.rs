use std::io::{Error, ErrorKind};
use tower_http::trace::TraceLayer;

use crate::{
    configuration::Settings,
    repository::{
        postgres::{get_connection_pool, ChatAdapter, CredentialsAdapter},
        redis::{get_redis_pool, TokenAdapter},
    },
    router::api::get_api_router,
    service::{AuthServiceImp, ChatServiceImp},
};

pub struct Application {
    listener: std::net::TcpListener,
    router: axum::Router,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = std::net::TcpListener::bind(address)?;
        let connection_pool = get_connection_pool(&configuration.database);
        let redis_pool = get_redis_pool(&configuration.redis);

        let cred_repo = CredentialsAdapter::new(connection_pool.clone());
        let chat_repo = ChatAdapter::new(connection_pool.clone());
        let token_repo = TokenAdapter::new(redis_pool);

        let chat_service = ChatServiceImp::new(chat_repo);
        let auth_service = AuthServiceImp::build(&configuration.auth, cred_repo, token_repo)?;
        let router = axum::Router::new()
            .nest("/api", get_api_router(auth_service, chat_service))
            .layer(TraceLayer::new_for_http());

        Ok(Self { listener, router })
    }

    pub async fn run_until_stopped(self) -> std::io::Result<()> {
        axum::Server::from_tcp(self.listener)
            .map_err(|err| Error::new(ErrorKind::Other, format!("listen error:{}", err)))?
            .serve(self.router.into_make_service())
            .await
            .map_err(|err| Error::new(ErrorKind::Other, format!("serve error:{}", err)))
    }
}
