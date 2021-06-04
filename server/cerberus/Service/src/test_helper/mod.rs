use testcontainers::clients::*;
use testcontainers::images::redis::Redis;
use testcontainers::{images, Container, Docker};
use tokio::task::JoinHandle;
use tokio::time::Duration;

#[cfg(not(feature = "test-helper"))]
use games_cheetah_cerberus_service::{
    server::*, service::storage::RedisRefreshTokenStorage, service::token::JWTTokensService,
};

#[cfg(feature = "test-helper")]
use crate::{
    server::*, service::storage::RedisRefreshTokenStorage, service::token::JWTTokensService,
};

pub const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEVVHNXKxoUNkoX9hnOJpSz6K2KDfi
gxaSXu+FIpP32qvcDgZPZ01tjnGjOysyPxRoZaMu/d9rHi3ulbceoYwS+Q==
-----END PUBLIC KEY-----";

pub const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgcg7dsJWSz8f40cEv
BTeGSzANXGlEzutd9IIm6/inl0ahRANCAARVUc1crGhQ2Shf2Gc4mlLPorYoN+KD
FpJe74Uik/faq9wOBk9nTW2OcaM7KzI/FGhloy7932seLe6Vtx6hjBL5
-----END PRIVATE KEY-----";

pub fn stub_token_service<'a>(
    session_exp: u64,
    refresh_exp: u64,
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
    time_of_life_in_sec: u64,
) -> (Container<'a, Cli, Redis>, RedisRefreshTokenStorage) {
    let node = (*CLI).run(images::redis::Redis::default());
    let port = node.get_host_port(6379).unwrap();
    (
        node,
        RedisRefreshTokenStorage::new(
            "127.0.0.1".to_owned(),
            port,
            Option::None,
            time_of_life_in_sec,
        )
        .unwrap(),
    )
}

pub async fn stub_cerberus_grpc_server<'a>(
    internal_port: u16,
    external_port: u16,
) -> (JoinHandle<()>, Container<'a, Cli, Redis>) {
    let node = (*CLI).run(images::redis::Redis::default());
    let port = node.get_host_port(6379).unwrap();
    let handler = tokio::spawn(async move {
        run_grpc_server(
            PUBLIC_KEY.to_owned(),
            PRIVATE_KEY.to_owned(),
            "127.0.0.1".to_owned(),
            port,
            Option::None,
            internal_port,
            external_port,
        )
        .await;
    });

    tokio::time::sleep(Duration::from_secs(1)).await;
    (handler, node)
}
