use std::future::Future;

use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use cheetah_libraries_microservice::trace::trace_err;

use crate::grpc::userstore::{
	self, fetch_server::Fetch, FetchDoubleReply, FetchDoubleRequest, FetchLongReply,
	FetchLongRequest, FetchStringReply, FetchStringRequest,
};
use crate::grpc::verify_credentials;
use crate::storage;

pub struct FetchService {
	fetcher: storage::Fetcher,
	jwt_public_key: String,
}

impl FetchService {
	pub fn new(pg_pool: PgPool, jwt_public_key: String) -> Self {
		Self {
			fetcher: storage::Fetcher::new(pg_pool),
			jwt_public_key,
		}
	}

	async fn process_request<T, R, V, Op, Fut>(
		&self,
		request: Request<T>,
		op: Op,
	) -> Result<Response<R>, Status>
	where
		R: From<userstore::FetchStatus> + From<V>,
		Op: FnOnce(Uuid, T) -> Fut,
		Fut: Future<Output = Result<Option<V>, sqlx::Error>>,
	{
		match verify_credentials(request, &self.jwt_public_key) {
			Ok((user, args)) => match op(user, args).await {
				Ok(value) => match value {
					None => Ok(Response::new(userstore::FetchStatus::FieldNotFound.into())),
					Some(value) => Ok(Response::new(value.into())),
				},
				Err(e) => {
					trace_err("Fetch operation failed", &e);
					Err(Status::internal("Internal error"))
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
			self.fetcher.get::<f64>(&user, &args.field_name).await
		})
		.await
	}

	async fn long(
		&self,
		request: Request<FetchLongRequest>,
	) -> Result<Response<FetchLongReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.fetcher.get::<i64>(&user, &args.field_name).await
		})
		.await
	}

	async fn string(
		&self,
		request: Request<FetchStringRequest>,
	) -> Result<Response<FetchStringReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.fetcher.get::<String>(&user, &args.field_name).await
		})
		.await
	}
}
