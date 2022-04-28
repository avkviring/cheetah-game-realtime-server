use cheetah_accounts::grpc::run_grpc_server;
use cheetah_accounts::postgresql::create_postgres_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("statistics-event-receiver");
	Ok(())
}
