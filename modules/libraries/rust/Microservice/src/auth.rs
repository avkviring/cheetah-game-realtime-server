use std::str::FromStr;

use jwt_tonic_user_uuid::JWTUserTokenParser;
use tonic::{
	metadata::{AsciiMetadataValue, MetadataMap},
	service::Interceptor,
	Request, Status,
};
use uuid::Uuid;

use crate::trace::trace_err;

pub const USER_KEY: &str = "user-bin";

#[derive(Clone)]
pub struct JwtAuthInterceptor {
	jwt_public_key: String,
}

impl JwtAuthInterceptor {
	pub fn new(jwt_public_key: String) -> Self {
		JwtAuthInterceptor { jwt_public_key }
	}
}

impl Interceptor for JwtAuthInterceptor {
	fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
		let parser = JWTUserTokenParser::new(self.jwt_public_key.clone());
		match parser.get_user_uuid_from_grpc(request.metadata()) {
			Err(e) => {
				trace_err("Unauthorized access attempt", e);
				Err(Status::permission_denied(""))
			}
			Ok(uuid) => {
				let mut md = request.metadata_mut();
				store_user_uuid(&mut md, &uuid);
				Ok(request)
			}
		}
	}
}

fn store_user_uuid(metadata: &mut MetadataMap, uuid: &Uuid) {
	let uuid_str = uuid.as_simple().to_string();
	metadata.insert(USER_KEY, AsciiMetadataValue::from_str(&uuid_str).unwrap());
}

pub fn load_user_uuid(metadata: &MetadataMap) -> Uuid {
	let uuid = metadata.get(USER_KEY).unwrap();
	let uuid_str = uuid.to_str().unwrap();
	Uuid::from_str(uuid_str).unwrap()
}

// pub fn verify_credentials<T>(
// 	request: Request<T>,
// 	jwt_public_key: &str,
// ) -> Result<(Uuid, T), Status> {
// 	let parser = JWTUserTokenParser::new(jwt_public_key.to_string());
// 	match parser.get_user_uuid_from_grpc(request.metadata()) {
// 		Err(e) => {
// 			trace_err("Unauthorized access attempt", e);
// 			Err(Status::permission_denied(""))
// 		}
// 		Ok(user) => {
// 			let args = request.into_inner();
// 			Ok((user, args))
// 		}
// 	}
// }
