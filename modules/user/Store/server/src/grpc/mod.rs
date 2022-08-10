use std::{error::Error, net::SocketAddr};

use jwt_tonic_user_uuid::JWTUserTokenParser;
use sqlx::PgPool;

use tonic::{transport::Server, Request, Status};
use uuid::Uuid;

use cheetah_libraries_microservice::{init, trace::trace_err};
use update::UpdateService;
use userstore::update_server::UpdateServer;

use self::{fetch::FetchService, userstore::fetch_server::FetchServer};

mod fetch;
mod reply;
mod update;
mod value;
mod userstore {
	tonic::include_proto!("cheetah.user.store.external");
}

pub struct Service {
	pg_pool: PgPool,
	jwt_public_key: String,
}

impl Service {
	pub fn new(pg_pool: PgPool, jwt_public_key: String) -> Self {
		Self {
			pg_pool,
			jwt_public_key,
		}
	}

	pub async fn serve(&self, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
		init("user.store");

		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

		let update_service = UpdateService::new(self.pg_pool.clone(), self.jwt_public_key.clone());
		health_reporter
			.set_serving::<UpdateServer<UpdateService>>()
			.await;

		let fetch_service = FetchService::new(self.pg_pool.clone(), self.jwt_public_key.clone());
		health_reporter
			.set_serving::<FetchServer<FetchService>>()
			.await;

		Server::builder()
			.accept_http1(true)
			.add_service(tonic_web::enable(health_service))
			.add_service(tonic_web::enable(UpdateServer::new(update_service)))
			.add_service(tonic_web::enable(FetchServer::new(fetch_service)))
			.serve(addr)
			.await?;

		Ok(())
	}
}

fn verify_credentials<T>(request: Request<T>, jwt_public_key: &str) -> Result<(Uuid, T), Status> {
	let parser = JWTUserTokenParser::new(jwt_public_key.to_string());
	match parser.get_user_uuid_from_grpc(request.metadata()) {
		Err(e) => {
			trace_err("Unauthorized access attempt", e);
			Err(Status::permission_denied(""))
		}
		Ok(user) => {
			let args = request.into_inner();
			Ok((user, args))
		}
	}
}
