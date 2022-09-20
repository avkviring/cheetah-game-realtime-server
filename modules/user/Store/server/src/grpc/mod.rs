mod fetch;
mod reply;
mod update;
mod value;
mod userstore {
	tonic::include_proto!("cheetah.user.store.external");
}

use std::{error::Error, net::SocketAddr};

use cheetah_libraries_microservice::auth::JwtAuthInterceptor;
use sqlx::PgPool;
use tonic::transport::Server;
use tonic_health::ServingStatus;

use fetch::FetchService;
use update::UpdateService;
use userstore::{fetch_server::FetchServer, update_server::UpdateServer};

pub struct Service {
	pg_pool: PgPool,
	jwt_public_key: String,
}

impl Service {
	pub fn new(pg_pool: PgPool, jwt_public_key: String) -> Self {
		Self { pg_pool, jwt_public_key }
	}

	pub async fn serve(&self, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

		let updater_service = UpdateService::new(self.pg_pool.clone());

		health_reporter.set_service_status("", ServingStatus::Serving).await;

		health_reporter.set_serving::<UpdateServer<UpdateService>>().await;

		let fetcher_service = FetchService::new(self.pg_pool.clone());
		health_reporter.set_serving::<FetchServer<FetchService>>().await;

		let auth_interceptor = JwtAuthInterceptor::new(self.jwt_public_key.to_owned());
		let updater_service = UpdateServer::with_interceptor(updater_service, auth_interceptor.clone());
		let fetcher_service = FetchServer::with_interceptor(fetcher_service, auth_interceptor.clone());

		Server::builder()
			.accept_http1(true)
			.add_service(tonic_web::enable(health_service))
			.add_service(tonic_web::enable(updater_service))
			.add_service(tonic_web::enable(fetcher_service))
			.serve(addr)
			.await?;

		Ok(())
	}
}
