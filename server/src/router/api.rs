use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
};

use crate::service;

use super::{auth, ws};

pub fn get_api_router<A, C>(auth_service: A, chat_service: C) -> axum::Router
where
    A: service::AuthService + Sync + Send + 'static,
    C: service::ChatService + Sync + Send + 'static,
{
    let auth_service = Arc::new(auth_service);
    let chat_service = Arc::new(chat_service);

    let require_authentication_middleware =
        middleware::from_fn_with_state(auth_service.clone(), auth::require_authentication);

    let chat_state = Arc::new(ws::ChatState::new(auth_service.clone(), chat_service));

    let chat_router = axum::Router::new()
        .route("/ws/:room", get(ws::websocket_handler))
        .with_state(chat_state);

    let auth_routes = axum::Router::new()
        .route("/logout", get(auth::logout))
        .route("/refresh", get(auth::refresh))
        .route_layer(require_authentication_middleware)
        .route("/login", post(auth::login))
        .route("/signup", post(auth::signup))
        .with_state(auth_service);

    axum::Router::new().merge(auth_routes).merge(chat_router)
}
