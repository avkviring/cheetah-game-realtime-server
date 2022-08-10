use std::future::Future;

use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use cheetah_libraries_microservice::trace::trace_err;

use crate::grpc::userstore::{update_server::Update, SetDoubleRequest, UpdateReply};
use crate::grpc::userstore::{SetLongRequest, SetStringRequest};
use crate::grpc::verify_credentials;
use crate::storage;

pub struct UpdateService {
	updater: storage::Updater,
	jwt_public_key: String,
}

impl UpdateService {
	pub fn new(pg_pool: PgPool, jwt_public_key: String) -> Self {
		Self {
			updater: storage::Updater::new(pg_pool),
			jwt_public_key,
		}
	}

	fn new_response(
		&self,
		result: Result<(), sqlx::Error>,
	) -> Result<Response<UpdateReply>, Status> {
		match result {
			Ok(_) => Ok(Response::new(UpdateReply::default())),
			Err(e) => {
				trace_err("Update operation failed", &e);
				Err(Status::internal("Internal error"))
			}
		}
	}

	async fn process_request<T, Fut>(
		&self,
		request: Request<T>,
		op: impl FnOnce(Uuid, T) -> Fut,
	) -> Result<Response<UpdateReply>, Status>
	where
		Fut: Future<Output = Result<(), sqlx::Error>>,
	{
		match verify_credentials(request, &self.jwt_public_key) {
			Err(s) => Err(s),
			Ok((user, args)) => self.new_response(op(user, args).await),
		}
	}
}

#[tonic::async_trait]
impl Update for UpdateService {
	async fn increment_double(
		&self,
		request: Request<SetDoubleRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.updater
				.increment(&user, &args.field_name, args.value)
				.await
		})
		.await
	}

	async fn increment_long(
		&self,
		request: Request<SetLongRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.updater
				.increment(&user, &args.field_name, args.value)
				.await
		})
		.await
	}

	async fn set_long(
		&self,
		request: Request<SetLongRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.updater.set(&user, &args.field_name, args.value).await
		})
		.await
	}

	async fn set_double(
		&self,
		request: Request<SetDoubleRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.updater.set(&user, &args.field_name, args.value).await
		})
		.await
	}

	async fn set_string(
		&self,
		request: Request<SetStringRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.updater.set(&user, &args.field_name, args.value).await
		})
		.await
	}
}
