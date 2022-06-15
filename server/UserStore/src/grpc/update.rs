use crate::grpc::userstore::{update_server::Update, SetIntRequest, UpdateReply};
use crate::ydb::YDBUpdate;
use cheetah_libraries_microservice::jwt::grpc::get_user_uuid;
use tonic::{Request, Response, Status};
use ydb::TableClient;

pub struct UpdateService {
	update_wrapper: YDBUpdate,
	jwt_public_key: String,
}

impl UpdateService {
	pub fn new(client: TableClient, jwt_public_key: String) -> Self {
		Self {
			update_wrapper: YDBUpdate::new(client),
			jwt_public_key,
		}
	}
}

#[tonic::async_trait]
impl Update for UpdateService {
	async fn set_int(
		&self,
		request: Request<SetIntRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		get_user_uuid(request.metadata(), self.jwt_public_key.clone())
			.map(|uuid| Response::new(UpdateReply::default()))
			.or_else(|e| Err(Status::permission_denied("Unauthorized")))
	}
}
