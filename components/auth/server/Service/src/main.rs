use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

use games_cheetah_auth_service::storage::storage::Storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let pg_user = std::env::var("PG_USER").expect("Env PG_USER not set");
    let pg_passwd = std::env::var("PG_PASSWD").expect("Env PG_PASSWD not set");
    let pg_host = std::env::var("PG_HOST").expect("Env PG_HOST not set");
    let pg_port = std::env::var("PG_PORT").expect("Env PG_PORT not set");

    let storage = Storage::new(
        pg_user.as_str(),
        pg_passwd.as_str(),
        pg_host.as_str(),
        pg_port.parse().unwrap(),
    )
    .await;

    let row: Vec<(String,)> = sqlx::query_as("SELECT id from players.players")
        .fetch_all(&storage.pool)
        .await?;

    println!("{:?}", row);

    Ok(())
}
