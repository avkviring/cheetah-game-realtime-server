use tonic::{Request, Response, Status};
use ydb::TableClient;

use crate::grpc::unwrap_request;
use crate::grpc::userstore::{
	update_server::Update, SetDoubleRequest, SetLongRequest, SetStringRequest, UpdateReply,
};
use crate::ydb::YDBUpdate;
use cheetah_libraries_microservice::trace::trace;

pub struct UpdateService {
	update: YDBUpdate,
	jwt_public_key: String,
}

impl UpdateService {
	pub fn new(client: TableClient, jwt_public_key: String) -> Self {
		Self {
			update: YDBUpdate::new(client),
			jwt_public_key,
		}
	}
}

#[tonic::async_trait]
impl Update for UpdateService {
	async fn set_long(
		&self,
		request: Request<SetLongRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(s) => Err(s),
			Ok((user, args)) => match self.update.set(&user, &args.field_name, &args.value).await {
				Ok(_) => Ok(Response::new(UpdateReply::default())),
				Err(e) => {
					trace("Update::set_long failed", &e);
					e.lift(|s| UpdateReply { status: s as i32 })
				}
			},
		}
	}

	async fn increment_long(
		&self,
		request: Request<SetLongRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(s) => Err(s),
			Ok((user, args)) => match self
				.update
				.increment(&user, &args.field_name, &args.value)
				.await
			{
				Ok(_) => Ok(Response::new(UpdateReply::default())),
				Err(e) => {
					trace("Update::increment_long failed", &e);
					e.lift(|s| UpdateReply { status: s as i32 })
				}
			},
		}
	}

	async fn set_double(
		&self,
		request: Request<SetDoubleRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(s) => Err(s),
			Ok((user, args)) => match self.update.set(&user, &args.field_name, &args.value).await {
				Ok(_) => Ok(Response::new(UpdateReply::default())),
				Err(e) => {
					trace("Update::set_double failed", &e);
					e.lift(|s| UpdateReply { status: s as i32 })
				}
			},
		}
	}

	async fn increment_double(
		&self,
		request: Request<SetDoubleRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(s) => Err(s),
			Ok((user, args)) => match self
				.update
				.increment(&user, &args.field_name, &args.value)
				.await
			{
				Ok(_) => Ok(Response::new(UpdateReply::default())),
				Err(e) => {
					trace("Update::increment_double failed", &e);
					e.lift(|s| UpdateReply { status: s as i32 })
				}
			},
		}
	}

	async fn set_string(
		&self,
		request: Request<SetStringRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(s) => Err(s),
			Ok((user, args)) => match self.update.set(&user, &args.field_name, &args.value).await {
				Ok(_) => Ok(Response::new(UpdateReply::default())),
				Err(e) => {
					trace("Update::set_string failed", &e);
					e.lift(|s| UpdateReply { status: s as i32 })
				}
			},
		}
	}
}
