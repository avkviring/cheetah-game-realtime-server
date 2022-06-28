use crate::grpc::userstore::{
	update_server::Update, SetDoubleRequest, SetLongRequest, UpdateReply,
};
use crate::ydb::YDBUpdate;
use tonic::{Request, Response, Status};
use ydb::TableClient;

use super::unwrap_request;

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
			Err(_) => Err(Status::permission_denied("")),
			Ok((user, args)) => match self.update.set(&user, &args.field_name, &args.value).await {
				Ok(_) => Ok(Response::new(UpdateReply::default())),
				Err(e) => Err(e.to_status(&args.field_name)),
			},
		}
	}

	async fn set_double(
		&self,
		request: Request<SetDoubleRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(_) => Err(Status::permission_denied("")),
			Ok((user, args)) => match self.update.set(&user, &args.field_name, &args.value).await {
				Ok(_) => Ok(Response::new(UpdateReply::default())),
				Err(e) => Err(e.to_status(&args.field_name)),
			},
		}
	}
}
