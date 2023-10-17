use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use shared::domain::UserId;
use uuid::Uuid;

pub const ALGORITHM: Algorithm = Algorithm::EdDSA;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Claims {
    sub: Uuid,
    token_id: Uuid,
    exp: u64,
}

impl Claims {
    pub fn user_id(&self) -> UserId {
        self.sub.into()
    }
    pub fn token_id(&self) -> Uuid {
        self.token_id
    }
}

pub fn encode_token(
    user_id: &UserId,
    token_id: Uuid,
    jwt_key: &EncodingKey,
    sec: u64,
) -> Result<Secret<String>, anyhow::Error> {
    let claims = Claims {
        sub: *user_id.as_ref(),
        token_id,
        exp: jsonwebtoken::get_current_timestamp() + sec,
    };
    let header = Header {
        alg: ALGORITHM,
        ..Default::default()
    };
    Ok(Secret::new(encode(&header, &claims, jwt_key)?))
}

pub fn decode_token(token: &str, jwt_key: &DecodingKey) -> Result<Claims, anyhow::Error> {
    let validation = Validation::new(ALGORITHM);
    let decoded = decode::<Claims>(token, jwt_key, &validation)?;
    Ok(decoded.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
    use uuid::Uuid;

    #[test]
    fn encode_decode_token() {
        let sub = Uuid::new_v4();
        let token_id = Uuid::new_v4();
        let (encoding_key, decoding_key) = generate_keys();

        let token = encode_token(&sub.into(), token_id, &encoding_key, 20).unwrap();
        let decoded = decode_token(token.expose_secret(), &decoding_key);

        let decoded = decoded.expect("Failed to decode token");
        assert_eq!(decoded.sub, sub);
    }

    fn generate_keys() -> (EncodingKey, DecodingKey) {
        let key_pair = jwt_simple::algorithms::Ed25519KeyPair::generate();
        dbg!(&key_pair.to_pem().to_string());
        dbg!(&key_pair.public_key().to_pem().to_string());
        let encoding_key = EncodingKey::from_ed_pem(key_pair.to_pem().as_bytes()).unwrap();
        let decoding_key =
            DecodingKey::from_ed_pem(key_pair.public_key().to_pem().as_bytes()).unwrap();
        (encoding_key, decoding_key)
    }
}
