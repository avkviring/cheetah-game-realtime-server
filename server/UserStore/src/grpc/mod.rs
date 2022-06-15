mod fetch;
mod update;
mod userstore {
	tonic::include_proto!("cheetah.userstore.external");
}

use std::{error::Error, net::SocketAddr};
use tonic::transport::Server;
use tonic_web;
use update::UpdateService;
use userstore::update_server::UpdateServer;
use ydb::Client;

pub struct Service {
	ydb_client: Client,
	jwt_public_key: String,
}

impl Service {
	pub async fn serve(&self, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
		let update_service =
			UpdateService::new(self.ydb_client.table_client(), self.jwt_public_key.clone());

		Server::builder()
			.accept_http1(true)
			.add_service(tonic_web::enable(UpdateServer::new(update_service)))
			.serve(addr)
			.await?;

		Ok(())
	}
}
