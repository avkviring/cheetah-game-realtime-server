use crate::grpc::userstore::{update_server::Update, SetIntRequest, UpdateReply};
use crate::ydb::YDBUpdate;
use cheetah_libraries_microservice::jwt::grpc::get_user_uuid;
use tonic::{Request, Response, Status};
use ydb::TableClient;

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
	async fn set_int(
		&self,
		request: Request<SetIntRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		let r = get_user_uuid(request.metadata(), self.jwt_public_key.clone());
		if let Err(_) = r {
			return Err(Status::permission_denied("Unauthorized"));
		}

		let user_id = r.unwrap();
		let args = request.into_inner();
		let r = self
			.update
			.set_int(&user_id, &args.field_name, args.value)
			.await;
		if let Err(e) = r {
			return Err(Status::unknown(e.to_string()));
		}

		Ok(Response::new(UpdateReply::default()))
	}
}
