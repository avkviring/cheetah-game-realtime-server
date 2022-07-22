use std::future::Future;

use ::ydb::TableClient;
use cheetah_libraries_microservice::trace::trace_err;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::grpc::userstore::{
	self, fetch_server::Fetch, FetchDoubleReply, FetchDoubleRequest, FetchLongReply,
	FetchLongRequest, FetchStringReply, FetchStringRequest,
};
use crate::grpc::verify_credentials;
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

	async fn process_request<T, R, V, Op, Fut>(
		&self,
		request: Request<T>,
		op: Op,
	) -> Result<Response<R>, Status>
	where
		R: From<userstore::Status> + From<V>,
		Op: FnOnce(Uuid, T) -> Fut,
		Fut: Future<Output = Result<V, ydb::Error>>,
	{
		match verify_credentials(request, &self.jwt_public_key) {
			Ok((user, args)) => match op(user, args).await {
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

#[tonic::async_trait]
impl Fetch for FetchService {
	async fn double(
		&self,
		request: Request<FetchDoubleRequest>,
	) -> Result<Response<FetchDoubleReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.fetch.get_double(&user, &args.field_name).await
		})
		.await
	}

	async fn long(
		&self,
		request: Request<FetchLongRequest>,
	) -> Result<Response<FetchLongReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.fetch.get_long(&user, &args.field_name).await
		})
		.await
	}

	async fn string(
		&self,
		request: Request<FetchStringRequest>,
	) -> Result<Response<FetchStringReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.fetch.get_string(&user, &args.field_name).await
		})
		.await
	}
}
