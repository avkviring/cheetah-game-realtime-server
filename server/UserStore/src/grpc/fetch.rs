use ::ydb::TableClient;
use cheetah_libraries_microservice::trace::trace_err;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::grpc::unwrap_request;
use crate::grpc::userstore::PrimitiveType;
use crate::grpc::userstore::{self, fetch_server::Fetch, FetchReply, FetchRequest};
use crate::ydb;

pub struct FetchService {
	fetch: ydb::Fetch,
	jwt_public_key: String,
}

impl FetchService {
	pub fn new(client: TableClient, jwt_public_key: String) -> Self {
		Self {
			fetch: ydb::Fetch::new(client),
			jwt_public_key,
		}
	}

	async fn fetch_wrapped(
		&self,
		user: &Uuid,
		args: &FetchRequest,
	) -> Result<userstore::PrimitiveValue, ydb::Error> {
		let field_name = &args.field_name;
		let fetch = &self.fetch;
		let datatype = args.datatype;

		if datatype == PrimitiveType::Double as i32 {
			fetch.get::<f64>(user, field_name).await.map(|v| v.into())
		} else if datatype == PrimitiveType::Long as i32 {
			fetch.get::<i64>(user, field_name).await.map(|v| v.into())
		} else if datatype == PrimitiveType::String as i32 {
			fetch
				.get::<String>(user, field_name)
				.await
				.map(|v| v.into())
		} else {
			panic!("Unhandled PrimitiveType variant")
		}
	}
}

#[tonic::async_trait]
impl Fetch for FetchService {
	async fn get(&self, request: Request<FetchRequest>) -> Result<Response<FetchReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Ok((user, args)) => match self.fetch_wrapped(&user, &args).await {
				Ok(value) => Ok(Response::new(value.into())),
				Err(e) => {
					if e.is_server_side() {
						trace_err("Fetch operation failed", &e);
					}
					e.lift(|s| s.into())
				}
			},
			Err(e) => Err(e),
		}
	}
}
