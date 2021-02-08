use testcontainers::clients::*;
use testcontainers::images::redis::Redis;
use testcontainers::{images, Container, Docker};

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

#[cfg(test)]
pub fn stub_token_service<'a>(
    session_exp: i64,
    refresh_exp: i64,
) -> (Container<'a, Cli, Redis>, JWTTokensService) {
    let (node, storage) = stub_storage(refresh_exp + 1);

    let service = JWTTokensService::new(
        PRIVATE_KEY.to_string(),
        PUBLIC_KEY.to_string(),
        session_exp,
        refresh_exp,
        storage,
    );
    (node, service)
}

lazy_static::lazy_static! {
    static ref CLI: Cli = Default::default();
}

fn stub_storage<'a>(
    time_of_life_in_sec: i64,
) -> (Container<'a, Cli, Redis>, RedisRefreshTokenStorage) {
    let node = (*CLI).run(images::redis::Redis::default());
    let port = node.get_host_port(6379).unwrap();
    (
        node,
        RedisRefreshTokenStorage::new("127.0.0.1".to_owned(), port, time_of_life_in_sec).unwrap(),
    )
}
