use anyhow::Context;

use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use shared::domain::{NewUser, User, UserCode, UserEmail, UserId, UserName};
use uuid::Uuid;

use crate::configuration::AuthSettings;
use crate::service::Error;

use super::{compute_password_hash, decode_token, encode_token, spawn_blocking_with_tracing};
use super::{verify_password_hash, Claims};

pub struct Credentials {
    pub email: String,
    pub password: Secret<String>,
}

type Tokens = (Secret<String>, Secret<String>);

#[derive(Clone)]
pub struct AuthServiceImp<CredRepo, TokenRepo>
where
    CredRepo: CredentialsRepository,
    TokenRepo: TokenRepository,
{
    credentials_repo: CredRepo,
    tokens_repo: TokenRepo,
    encoding_key: jsonwebtoken::EncodingKey,
    decoding_key: jsonwebtoken::DecodingKey,
    access_toked_duration: u64,
    refresh_toked_duration: u64,
}

impl<CredRepo, TokenRepo> AuthServiceImp<CredRepo, TokenRepo>
where
    CredRepo: CredentialsRepository,
    TokenRepo: TokenRepository,
{
    pub fn build(
        config: &AuthSettings,
        credentials_repo: CredRepo,
        tokens_repo: TokenRepo,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            credentials_repo,
            tokens_repo,
            encoding_key: config.encoding_key()?,
            decoding_key: config.decoding_key()?,
            access_toked_duration: config.access_toked_duration,
            refresh_toked_duration: config.refresh_toked_duration,
        })
    }
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait CredentialsRepository {
    async fn get_credential(
        &self,
        email: &str,
    ) -> Result<Option<(Uuid, Secret<String>)>, anyhow::Error>;

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error>;

    async fn signup(
        &self,
        user_name: &UserName,
        user_email: &UserEmail,
        user_password_hash: Secret<String>,
        code: &UserCode,
    ) -> Result<User, anyhow::Error>;
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait TokenRepository {
    async fn create(&self, token_id: &Uuid, user_id: &UserId) -> anyhow::Result<()>;
    async fn exist(&self, token_id: &Uuid, user_id: &UserId) -> anyhow::Result<Option<UserId>>;
    async fn delete(&self, token_id: &Uuid, user_id: &UserId) -> anyhow::Result<()>;
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait AuthService {
    async fn validate_token(&self, token: &str) -> Result<Claims, Error>;
    async fn signup(&self, new_user: NewUser) -> Result<(User, Tokens), Error>;
    async fn login(&self, credentials: Credentials) -> Result<Tokens, Error>;
    async fn logout(&self, access_token: Secret<String>) -> Result<(), Error>;
    async fn refresh(&self, refresh_token: Secret<String>) -> Result<Tokens, Error>;
}

#[async_trait]
impl<CredRepo, TokenRepo> AuthService for AuthServiceImp<CredRepo, TokenRepo>
where
    CredRepo: CredentialsRepository + Send + Sync,
    TokenRepo: TokenRepository + Send + Sync,
{
    #[tracing::instrument(name = "Login User", skip(self, credentials))]
    async fn login(&self, credentials: Credentials) -> Result<Tokens, Error> {
        let user_id = self.validate_credentials(credentials).await?;
        Ok(self.create_token_pair(&user_id).await?)
    }

    #[tracing::instrument(name = "Signup User", skip(self, new_user))]
    async fn signup(&self, new_user: NewUser) -> Result<(User, Tokens), Error> {
        let NewUser {
            name,
            email,
            code,
            password,
        } = new_user;

        if self
            .credentials_repo
            .get_user_by_email(email.as_ref())
            .await?
            .is_some()
        {
            return Err(Error::ConflictError("User already exist".to_string()));
        }

        let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
            .await
            .map_err(|e| Error::UnexpectedError(e.into()))??;
        // TODO:invalidate old refresh token
        let user = self
            .credentials_repo
            .signup(&name, &email, password_hash, &code)
            .await?;

        let user_id = user.user_id.clone();
        Ok((user, (self.create_token_pair(&user_id).await?)))
    }

    #[tracing::instrument(name = "Login User", skip(self, access_token))]
    async fn logout(&self, access_token: Secret<String>) -> Result<(), Error> {
        let claims = decode_token(access_token.expose_secret(), &self.decoding_key)
            .context("Invalid token.")
            .map_err(Error::InvalidCredentials)?;

        self.tokens_repo
            .delete(&claims.token_id(), &claims.user_id())
            .await
            .context("Failed delete refresh token.")?;

        Ok(())
    }

    #[tracing::instrument(name = "Login User", skip(self, refresh_token))]
    async fn refresh(&self, refresh_token: Secret<String>) -> Result<Tokens, Error> {
        let claims = decode_token(refresh_token.expose_secret(), &self.decoding_key)
            .context("Invalid token.")
            .map_err(Error::InvalidCredentials)?;

        let user_id = self
            .tokens_repo
            .exist(&claims.token_id(), &claims.user_id())
            .await
            .context("Unknown refresh token.")?
            .ok_or_else(|| anyhow::anyhow!("Unknown refresh token."))
            .map_err(Error::InvalidCredentials)?;
        // TODO:add rotation
        self.tokens_repo
            .delete(&claims.token_id(), &claims.user_id())
            .await
            .context("Failed delete refresh token.")?;

        Ok(self.create_token_pair(&user_id).await?)
    }

    async fn validate_token(&self, token: &str) -> Result<Claims, Error> {
        let claims = decode_token(token, &self.decoding_key)
            .context("Invalid token")
            .map_err(Error::InvalidCredentials)?;
        Ok(claims)
    }
}

impl<CredRepo, TokenRepo> AuthServiceImp<CredRepo, TokenRepo>
where
    CredRepo: CredentialsRepository + Send + Sync,
    TokenRepo: TokenRepository + Send + Sync,
{
    #[tracing::instrument(name = "Validate credentials", skip(self, credentials))]
    async fn validate_credentials(&self, credentials: Credentials) -> Result<UserId, Error> {
        let mut user_id = None;
        //dummy load
        let mut expected_password_hash = Secret::new(
            "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
                .to_string(),
        );

        if let Some((stored_user_id, stored_password_hash)) = self
            .credentials_repo
            .get_credential(&credentials.email)
            .await?
        {
            user_id = Some(stored_user_id);
            expected_password_hash = stored_password_hash;
        }

        spawn_blocking_with_tracing(move || {
            verify_password_hash(expected_password_hash, credentials.password)
        })
        .await
        .context("Failed to spawn blocking task.")??;

        Ok(user_id
            .ok_or_else(|| anyhow::anyhow!("Unknown username."))
            .map_err(Error::InvalidCredentials)?
            .into())
    }

    async fn create_token_pair(&self, user_id: &UserId) -> Result<Tokens, anyhow::Error> {
        let token_id = uuid::Uuid::new_v4();
        self.tokens_repo
            .create(&token_id, &user_id)
            .await
            .context("Failed to store refresh token.")?;

        let access_token = encode_token(
            user_id,
            token_id,
            &self.encoding_key,
            self.access_toked_duration,
        )
        .context("Failed encode access token.")?;

        let refresh_token = encode_token(
            user_id,
            token_id,
            &self.encoding_key,
            self.refresh_toked_duration,
        )
        .context("Failed encode refresh token.")?;

        Ok((access_token, refresh_token))
    }
}
