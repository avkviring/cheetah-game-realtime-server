use std::future::Future;

use ::ydb::TableClient;
use cheetah_libraries_microservice::trace::trace_err;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::grpc::userstore::{update_server::Update, SetDoubleRequest, UpdateReply};
use crate::grpc::userstore::{SetLongRequest, SetStringRequest};
use crate::grpc::verify_credentials;
use crate::ydb;

pub struct UpdateService {
	update: ydb::Update,
	jwt_public_key: String,
}

impl UpdateService {
	pub fn new(client: TableClient, jwt_public_key: String) -> Self {
		Self {
			update: ydb::Update::new(client),
			jwt_public_key,
		}
	}

	fn new_response(
		&self,
		result: Result<(), ydb::Error>,
	) -> Result<Response<UpdateReply>, Status> {
		match result {
			Ok(_) => Ok(Response::new(UpdateReply::default())),
			Err(e) => {
				if e.is_server_side() {
					trace_err("Update operation failed", &e);
				}
				e.lift(|s| UpdateReply { status: s as i32 })
			}
		}
	}

	async fn process_request<T, Fut>(
		&self,
		request: Request<T>,
		op: impl FnOnce(Uuid, T) -> Fut,
	) -> Result<Response<UpdateReply>, Status>
	where
		Fut: Future<Output = Result<(), ydb::Error>>,
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
			self.update
				.increment(&user, &args.field_name, &args.value)
				.await
		})
		.await
	}

	async fn increment_long(
		&self,
		request: Request<SetLongRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.update
				.increment(&user, &args.field_name, &args.value)
				.await
		})
		.await
	}

	async fn set_long(
		&self,
		request: Request<SetLongRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.update.set(&user, &args.field_name, &args.value).await
		})
		.await
	}

	async fn set_double(
		&self,
		request: Request<SetDoubleRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.update.set(&user, &args.field_name, &args.value).await
		})
		.await
	}

	async fn set_string(
		&self,
		request: Request<SetStringRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		self.process_request(request, |user, args| async move {
			self.update.set(&user, &args.field_name, &args.value).await
		})
		.await
	}
}
