use tonic::transport::Server;

use crate::service::cookie::CookieService;
use crate::service::google::GoogleService;
use crate::storage::pg::PgStorage;
use jsonwebtoken_google::Parser;

pub async fn run_grpc_server(
    storage: PgStorage,
    public_jwt_key: String,
    cerberus_url: &str,
    service_port: u16,
    enable_cookie: bool,
    google_token_parser: Option<Parser>,
) {
    let addr = format!("0.0.0.0:{}", service_port).parse().unwrap();
    let mut builder = Server::builder();

    let builder = builder.add_optional_service(if enable_cookie {
        Some(
            crate::proto::auth::external::cookie::cookie_server::CookieServer::new(
                CookieService::new(storage.clone(), cerberus_url),
            ),
        )
    } else {
        None
    });
    let builder =
        builder.add_optional_service(google_token_parser.map(|parser| {
            crate::proto::auth::external::google::google_server::GoogleServer::new(
                GoogleService::new(storage, cerberus_url, parser, public_jwt_key),
            )
        }));

    builder.serve(addr).await.unwrap();
}
