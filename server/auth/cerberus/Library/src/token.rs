use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

///
/// Валидация и получение идентификатора пользователя из сессионного токена
///
#[derive(Debug)]
pub struct JWTTokenParser {
    public_key: String,
}

///
/// JWT сессионный токен
///
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTokenClaims {
    pub exp: usize,
    pub player: u64,
}

#[derive(Debug)]
pub enum SessionTokenError {
    InvalidSignature,
    Expired,
}

impl JWTTokenParser {
    pub fn new(public_key: String) -> Self {
        Self { public_key }
    }

    pub fn get_player_id(&self, token: String) -> Result<u64, SessionTokenError> {
        let token = JWTTokenParser::add_head(token);
        match jsonwebtoken::decode::<SessionTokenClaims>(
            token.as_str(),
            &DecodingKey::from_ec_pem(self.public_key.as_bytes()).unwrap(),
            &Validation::new(Algorithm::ES256),
        ) {
            Ok(token) => Result::Ok(token.claims.player),
            Err(error) => match error.kind() {
                ErrorKind::ExpiredSignature => Result::Err(SessionTokenError::Expired),
                _ => Result::Err(SessionTokenError::InvalidSignature),
            },
        }
    }

    ///
    ///  "typ": "JWT",
    //   "alg": "ES256"
    //
    pub fn add_head(token: String) -> String {
        format!("eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiJ9.{}", token)
    }
}
