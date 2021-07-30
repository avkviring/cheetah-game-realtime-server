use std::time::{SystemTime, UNIX_EPOCH};

use super::storage::RedisRefreshTokenStorage;
use cheetah_microservice::jwt::{JWTTokenParser, SessionTokenClaims};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    exp: usize,
    player: u64,
    device_id: String,
    version: u64,
}

#[derive(Debug)]
pub struct Tokens {
    pub session: String,
    pub refresh: String,
}

#[derive(Debug)]
pub enum JWTTokensServiceError {
    InvalidSignature,
    Expired,
    InvalidId,
    StorageError(String),
}

pub struct JWTTokensService {
    session_exp_in_sec: u64,
    refresh_exp_in_sec: u64,
    private_key: String,
    public_key: String,
    storage: RedisRefreshTokenStorage,
}

impl JWTTokensService {
    pub fn new(
        private_key: String,
        public_key: String,
        session_exp_in_sec: u64,
        refresh_exp_in_sec: u64,
        storage: RedisRefreshTokenStorage,
    ) -> Self {
        Self {
            session_exp_in_sec,
            refresh_exp_in_sec,
            private_key,
            public_key,
            storage,
        }
    }

    pub async fn create(
        &self,
        player: u64,
        device_id: String,
    ) -> Result<Tokens, JWTTokensServiceError> {
        Result::Ok(Tokens {
            session: self.create_session_token(player),
            refresh: self.create_refresh_token(player, device_id).await?,
        })
    }

    async fn create_refresh_token(
        &self,
        player: u64,
        device_id: String,
    ) -> Result<String, JWTTokensServiceError> {
        let version = self
            .storage
            .new_version(player, &device_id)
            .await
            .map_err(|e| JWTTokensServiceError::StorageError(format!("{:?}", e)))?;

        let timestamp = JWTTokensService::get_time_stamp();
        let claims = RefreshTokenClaims {
            exp: (timestamp + self.refresh_exp_in_sec) as usize,
            player,
            device_id,
            version,
        };
        let token = encode(
            &Header::new(Algorithm::ES256),
            &claims,
            &EncodingKey::from_ec_pem(&self.private_key.as_bytes()).unwrap(),
        )
        .unwrap();
        Result::Ok(JWTTokensService::remove_head(token))
    }

    fn create_session_token(&self, player: u64) -> String {
        let timestamp = JWTTokensService::get_time_stamp();
        let claims = SessionTokenClaims {
            exp: (timestamp + self.session_exp_in_sec) as usize,
            player,
        };

        let token = encode(
            &Header::new(Algorithm::ES256),
            &claims,
            &EncodingKey::from_ec_pem(&self.private_key.as_bytes()).unwrap(),
        )
        .unwrap();
        JWTTokensService::remove_head(token)
    }

    fn get_time_stamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    fn remove_head(token: String) -> String {
        let collect: Vec<_> = token.split(".").collect();
        format!("{}.{}", collect[1], collect[2])
    }

    pub async fn refresh(&self, refresh_token: String) -> Result<Tokens, JWTTokensServiceError> {
        let token = JWTTokenParser::add_head(refresh_token);
        match jsonwebtoken::decode::<RefreshTokenClaims>(
            token.as_str(),
            &DecodingKey::from_ec_pem(self.public_key.as_bytes()).unwrap(),
            &Validation::new(Algorithm::ES256),
        ) {
            Ok(token) => {
                let player = token.claims.player;
                let device_id = token.claims.device_id;
                if self.storage.get_version(player, &device_id).await.unwrap()
                    == token.claims.version
                {
                    Result::Ok(Tokens {
                        session: self.create_session_token(player),
                        refresh: self.create_refresh_token(player, device_id).await?,
                    })
                } else {
                    Result::Err(JWTTokensServiceError::InvalidId)
                }
            }
            Err(error) => match error.kind() {
                ErrorKind::ExpiredSignature => Result::Err(JWTTokensServiceError::Expired),
                _ => Result::Err(JWTTokensServiceError::InvalidSignature),
            },
        }
    }
}
