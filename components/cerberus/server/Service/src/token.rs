use chrono::Utc;
use games_cheetah_cerberus_library::cerberus::SessionTokenClaims;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    exp: usize,
    user_id: String,
    device_id: String,
}

#[derive(Debug)]
pub struct Tokens {
    pub session: String,
    pub refresh: String,
}
pub struct JWTTokensService {
    pub session_exp_in_sec: i64,
    pub refresh_exp_in_sec: i64,
    pub private_key: String,
    pub public_key: String,
}

impl JWTTokensService {
    pub fn new(private_key: String, public_key: String) -> Self {
        Self {
            session_exp_in_sec: 5 * 60 * 60,       // 5 часов
            refresh_exp_in_sec: 30 * 24 * 60 * 60, // 1 месяц
            private_key,
            public_key,
        }
    }

    pub fn create_tokens(&self, user_id: String, device_id: String) -> Tokens {
        let session_claims = SessionTokenClaims {
            exp: (Utc::now().timestamp() + self.session_exp_in_sec) as usize,
            user_id,
        };

        let session_token = encode(
            &Header::new(Algorithm::ES256),
            &session_claims,
            &EncodingKey::from_ec_pem(&self.private_key.as_bytes()).unwrap(),
        )
        .unwrap();

        Tokens {
            session: JWTTokensService::remove_head(session_token),
            refresh: "".to_string(),
        }
    }

    fn remove_head(token: String) -> String {
        let collect: Vec<_> = token.split(".").collect();
        format!("{}.{}", collect[1], collect[2])
    }

    pub fn refresh_token(refresh_token: String) -> Tokens {
        Tokens {
            session: "".to_string(),
            refresh: "".to_string(),
        }
    }
}
