use ::ydb::TableClient;
use cheetah_libraries_microservice::trace::trace_err;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::grpc::unwrap_request;
use crate::grpc::userstore::{
	numeric_value, primitive_value, update_server::Update, IncrementRequest, SetRequest,
	UpdateReply,
};
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

	async fn set_wrapped(&self, user: &Uuid, args: SetRequest) -> Result<(), ydb::Error> {
		let field_name = &args.field_name;
		match args.value {
			Some(v) => match v.pr {
				Some(primitive_value::Pr::Long(v)) => self.update.set(user, field_name, &v).await,
				Some(primitive_value::Pr::Double(v)) => self.update.set(user, field_name, &v).await,
				Some(primitive_value::Pr::String(v)) => self.update.set(user, field_name, &v).await,
				None => panic!("Unknown variant of primitive_value::Pr"),
			},
			None => panic!("Empty field 'value'"),
		}
	}

	async fn increment_wrapped(
		&self,
		user: &Uuid,
		args: IncrementRequest,
	) -> Result<(), ydb::Error> {
		let field_name = &args.field_name;
		match args.value {
			Some(v) => match v.number {
				Some(numeric_value::Number::Long(v)) => {
					self.update.increment(user, field_name, &v).await
				}
				Some(numeric_value::Number::Double(v)) => {
					self.update.increment(user, field_name, &v).await
				}
				None => panic!("Unknown variant of numeric_value::Number"),
			},
			None => panic!("Empty field 'value'"),
		}
	}

	fn respond(&self, result: Result<(), ydb::Error>) -> Result<Response<UpdateReply>, Status> {
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
}

#[tonic::async_trait]
impl Update for UpdateService {
	async fn increment(
		&self,
		request: Request<IncrementRequest>,
	) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(s) => Err(s),
			Ok((user, args)) => self.respond(self.increment_wrapped(&user, args).await),
		}
	}

	async fn set(&self, request: Request<SetRequest>) -> Result<Response<UpdateReply>, Status> {
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Err(s) => Err(s),
			Ok((user, args)) => self.respond(self.set_wrapped(&user, args).await),
		}
	}
}
