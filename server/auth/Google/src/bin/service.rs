use cheetah_auth_google::{run_grpc_server, storage};
use cheetah_microservice::get_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cheetah_microservice::init("auth::Google");

    let pg_user = get_env("POSTGRES_USER");
    let pg_passwd = get_env("POSTGRES_PASSWORD");
    let pg_db = get_env("POSTGRES_DB");
    let pg_host = get_env("POSTGRES_HOST");
    let pg_port: u16 = get_env("POSTGRES_PORT").parse().unwrap();

    let google_token_parser = jsonwebtoken_google::Parser::new(&get_env("AUTH_GOOGLE_CLIENT_ID"));

    let jwt_public_key = get_env("JWT_PUBLIC_KEY");

    let pool = storage::create_postgres_pool(&pg_db, &pg_user, &pg_passwd, &pg_host, pg_port).await;

    storage::migrate_db(&pool).await;

    let cerberus_internal_service_uri =
        cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_AUTH_CERBERUS");
    let user_internal_service_uri =
        cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_AUTH_USER");

    run_grpc_server(
        pool,
        5000,
        cerberus_internal_service_uri,
        user_internal_service_uri,
        jwt_public_key,
        google_token_parser,
    )
    .await;

    Ok(())
}
