use axum::extract::State;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use axum::TypedHeader;
use secrecy::ExposeSecret;
use secrecy::Secret;
use shared::domain::NewUser;
use std::sync::Arc;

use crate::service;

use super::dto::LoginRequest;
use super::dto::TokensResponse;
use super::SignupRequest;
use super::SignupResponse;

#[tracing::instrument(name = "Login user", skip(auth_service, login_request))]
pub async fn login<A>(
    State(auth_service): State<Arc<A>>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Response, service::Error>
where
    A: service::AuthService,
{
    let (asses_token, refresh_token) = auth_service
        .login(service::Credentials::from(login_request))
        .await?;

    let token_response = TokensResponse {
        access_token: asses_token.expose_secret().to_string(),
        refresh_token: refresh_token.expose_secret().to_string(),
    };
    Ok((StatusCode::OK, Json(token_response)).into_response())
}

#[tracing::instrument(name = "Refresh token", skip(auth_service, bearer))]
pub async fn refresh<A>(
    State(auth_service): State<Arc<A>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Response, service::Error>
where
    A: service::AuthService,
{
    let token = Secret::new(bearer.token().to_string());
    let (asses_token, refresh_token) = auth_service.refresh(token).await?;

    let token_response = TokensResponse {
        access_token: asses_token.expose_secret().to_string(),
        refresh_token: refresh_token.expose_secret().to_string(),
    };
    Ok((StatusCode::OK, Json(token_response)).into_response())
}

#[tracing::instrument(name = "Logout user", skip(auth_service, bearer))]
pub async fn logout<A>(
    State(auth_service): State<Arc<A>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Response, service::Error>
where
    A: service::AuthService,
{
    let token = Secret::new(bearer.token().to_string());
    auth_service.logout(token).await?;

    Ok((StatusCode::NO_CONTENT).into_response())
}

#[tracing::instrument(name = "Signup user", skip(auth_service, req))]
pub async fn signup<A>(
    State(auth_service): State<Arc<A>>,
    Json(req): Json<SignupRequest>,
) -> Result<Response, service::Error>
where
    A: service::AuthService,
{
    let new_user = NewUser::try_from(req)?;

    let (user, (asses_token, refresh_token)) = auth_service.signup(new_user).await?;

    let response = SignupResponse {
        user: user.into(),
        access_token: asses_token.expose_secret().to_string(),
        refresh_token: refresh_token.expose_secret().to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}
