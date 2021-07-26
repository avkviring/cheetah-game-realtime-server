use cheetah_auth_cookie::{run_grpc_server, storage};
use cheetah_microservice::get_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cheetah_microservice::init("auth::Google");

    let pg_user = get_env("POSTGRES_USER");
    let pg_passwd = get_env("POSTGRES_PASSWORD");
    let pg_db = get_env("POSTGRES_DB");
    let pg_host = get_env("POSTGRES_HOST");
    let pg_port: u16 = get_env("POSTGRES_PORT").parse().unwrap();

    let cerberus_host = get_env("CERBERUS_INTERNAL_HOST");
    let cerberus_port = get_env("CERBERUS_INTERNAL_PORT");
    let user_host = get_env("USER_INTERNAL_HOST");
    let user_port = get_env("USER_INTERNAL_PORT");

    let cerberus_url = format!("http://{}:{}", cerberus_host, cerberus_port);
    let user_url = format!("http://{}:{}", user_host, user_port);

    let pool = storage::create_postgres_pool(&pg_db, &pg_user, &pg_passwd, &pg_host, pg_port).await;
    storage::migrate_db(&pool).await;

    run_grpc_server(pool, &cerberus_url, &user_url, 5000).await;

    Ok(())
}
