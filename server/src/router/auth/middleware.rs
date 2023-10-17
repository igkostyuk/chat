use crate::service;
use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::Request,
    middleware::Next,
    response::Response,
    TypedHeader,
};
use std::sync::Arc;

pub async fn require_authentication<T, A>(
    State(auth_service): State<Arc<A>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut request: Request<T>,
    next: Next<T>,
) -> Result<Response, service::Error>
where
    A: service::AuthService,
{
    let claims = auth_service.validate_token(bearer.token()).await?;
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
