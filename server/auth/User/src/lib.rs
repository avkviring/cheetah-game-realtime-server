pub mod proto;
pub mod storage;

use crate::proto::auth::user::internal::{user_server, CreateRequest, CreateResponse};
use crate::storage::Storage;
use tonic::{transport::Server, Request, Response};

pub async fn run_grpc_server(pool: sqlx::PgPool, service_port: u16) {
    Server::builder()
        .add_service(Service::new(pool).server())
        .serve(format!("0.0.0.0:{}", service_port).parse().unwrap())
        .await
        .unwrap();
}

pub struct Service {
    storage: Storage,
}

impl Service {
    pub fn new(storage: impl Into<Storage>) -> Self {
        let storage = storage.into();
        Self { storage }
    }

    pub fn server(self) -> user_server::UserServer<Self> {
        user_server::UserServer::new(self)
    }
}

#[tonic::async_trait]
impl user_server::User for Service {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, tonic::Status> {
        let ip = request.get_ref().ip.parse();
        let ip = ip.map_err(|_| tonic::Status::invalid_argument("ip address can't parsed"))?;
        let id = self.storage.create(ip).await.into();
        Ok(Response::new(CreateResponse { id }))
    }
}

#[cfg(test)]
pub mod test {
    use sqlx::PgPool;
    use std::collections::HashMap;
    use testcontainers::images::postgres::Postgres;
    use testcontainers::{clients::Cli, Container, Docker as _};

    pub async fn setup_postgresql_storage(cli: &Cli) -> (PgPool, Container<'_, Cli, Postgres>) {
        let mut env = HashMap::default();
        env.insert("POSTGRES_USER".to_owned(), "authentication".to_owned());
        env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
        let image = Postgres::default().with_version(13).with_env_vars(env);
        let node = cli.run(image);
        let port = node.get_host_port(5432).unwrap();
        let storage = crate::storage::create_postgres_pool(
            "authentication",
            "authentication",
            "passwd",
            "127.0.0.1",
            port,
        )
        .await;
        (storage, node)
    }
}
