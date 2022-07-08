use std::fmt::Debug;

use cheetah_libraries_microservice::trace::trace;
use tonic::{Request, Response, Status};
use ydb::{TableClient, Value};

use crate::grpc::request::RequestWithField;
use crate::grpc::unwrap_request;
use crate::grpc::userstore::{
	fetch_server::Fetch, GetDoubleReply, GetDoubleRequest, GetLongReply, GetLongRequest,
	GetStringReply, GetStringRequest, Status as UserStoreStatus,
};
use crate::ydb::primitive::Primitive;
use crate::ydb::YDBFetch;

pub struct FetchService {
	fetch: YDBFetch,
	jwt_public_key: String,
}

impl FetchService {
	pub fn new(client: TableClient, jwt_public_key: String) -> Self {
		Self {
			fetch: YDBFetch::new(client),
			jwt_public_key,
		}
	}

	async fn process_request<T, R, V>(
		&self,
		request: Request<T>,
		func_name: &str,
	) -> Result<Response<R>, Status>
	where
		T: RequestWithField,
		R: From<UserStoreStatus> + From<V>,
		V: Primitive,
		// Со слоя YDB протекает Value, это не идеальный вариант
		Option<V>: TryFrom<Value>,
		<Option<V> as TryFrom<Value>>::Error: Debug,
	{
		match unwrap_request(request, self.jwt_public_key.clone()) {
			Ok((user, args)) => match self.fetch.get::<V>(&user, &args.field_name()).await {
				Ok(value) => Ok(Response::new(value.into())),
				Err(e) => {
					trace(format!("Fetch::{} failed", func_name), &e);
					e.lift(|s| s.into())
				}
			},
			Err(e) => Err(e),
		}
	}
}

#[tonic::async_trait]
impl Fetch for FetchService {
	async fn get_long(
		&self,
		request: Request<GetLongRequest>,
	) -> Result<Response<GetLongReply>, Status> {
		self.process_request::<_, _, i64>(request, "get_long").await
	}

	async fn get_double(
		&self,
		request: Request<GetDoubleRequest>,
	) -> Result<Response<GetDoubleReply>, Status> {
		self.process_request::<_, _, f64>(request, "get_double")
			.await
	}

	async fn get_string(
		&self,
		request: Request<GetStringRequest>,
	) -> Result<Response<GetStringReply>, Status> {
		self.process_request::<_, _, String>(request, "get_string")
			.await
	}
}
