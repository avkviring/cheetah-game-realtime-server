use games_cheetah_cerberus_service::storage::*;
use games_cheetah_cerberus_service::token::*;

pub const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEVVHNXKxoUNkoX9hnOJpSz6K2KDfi
gxaSXu+FIpP32qvcDgZPZ01tjnGjOysyPxRoZaMu/d9rHi3ulbceoYwS+Q==
-----END PUBLIC KEY-----";

pub const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgcg7dsJWSz8f40cEv
BTeGSzANXGlEzutd9IIm6/inl0ahRANCAARVUc1crGhQ2Shf2Gc4mlLPorYoN+KD
FpJe74Uik/faq9wOBk9nTW2OcaM7KzI/FGhloy7932seLe6Vtx6hjBL5
-----END PRIVATE KEY-----";

pub struct StubRefreshTokenStorage {}

impl games_cheetah_cerberus_service::storage::Storage for StubRefreshTokenStorage {
    fn new_version(&mut self, user_id: &String, device_id: &String) -> u64 {
        0
    }

    fn get_version(&mut self, user_id: &String, device_id: &String) -> u64 {
        0
    }
}

pub fn new_service_with_stub_storage(
    session_exp: i64,
    refresh_exp: i64,
) -> JWTTokensService<StubRefreshTokenStorage> {
    let service = JWTTokensService::new(
        PRIVATE_KEY.to_string(),
        PUBLIC_KEY.to_string(),
        session_exp,
        refresh_exp,
        StubRefreshTokenStorage {},
    );
    service
}

pub fn new_service_with_redis_storage(
    session_exp: i64,
    refresh_exp: i64,
    port: u16,
) -> JWTTokensService<RedisRefreshTokenStorage> {
    let service = JWTTokensService::new(
        PRIVATE_KEY.to_string(),
        PUBLIC_KEY.to_string(),
        session_exp,
        refresh_exp,
        RedisRefreshTokenStorage::new("127.0.0.1".to_owned(), port, 1).unwrap(),
    );
    service
}
