use std::future::Future;

use tonic::transport::{Error, Server};

use crate::service::cookie::CookieService;
use crate::service::google::GoogleService;
use crate::storage::storage::Storage;

pub async fn run_grpc_server(
    storage: Storage,
    service_port: u16,
    cerberus_url: &str,
    parser: jsonwebtoken_google::Parser,
    public_jwt_key: String,
) {
    let addr = format!("0.0.0.0:{}", service_port).parse().unwrap();

    let cookie = crate::proto::auth::external::cookie::cookie_server::CookieServer::new(
        CookieService::new(storage.clone(), cerberus_url),
    );

    let google = crate::proto::auth::external::google::google_server::GoogleServer::new(
        GoogleService::new(storage, cerberus_url, parser, public_jwt_key),
    );
    Server::builder()
        .add_service(cookie)
        .add_service(google)
        .serve(addr)
        .await;
}
