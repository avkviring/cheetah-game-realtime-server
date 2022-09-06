use std::str::FromStr;

use jwt_tonic_user_uuid::JWTUserTokenParser;
use tonic::{
	metadata::{AsciiMetadataValue, MetadataMap},
	service::Interceptor,
	Request, Status,
};
use uuid::Uuid;

use crate::trace;

pub const USER_KEY: &str = "user";

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
				trace::err("Unauthorized access attempt", e);
				Err(Status::permission_denied(""))
			}
			Ok(uuid) => {
				let md = request.metadata_mut();
				store_user_uuid(md, &uuid);
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

#[cfg(test)]
mod test {
	use tonic::Code;

	use super::{store_user_uuid, AsciiMetadataValue, Interceptor, JwtAuthInterceptor, MetadataMap, Request, Uuid};

	#[test]
	fn test_store_user_uuid_does_not_panic() {
		let mut md = MetadataMap::new();
		let uuid = Uuid::new_v4();
		store_user_uuid(&mut md, &uuid)
	}

	#[test]
	#[should_panic(expected = "invalid metadata key")]
	fn test_bin_for_ascii() {
		let mut md = MetadataMap::new();
		md.append("yes-bin", AsciiMetadataValue::from_static("fail"));
	}

	#[test]
	fn test_no_bin_for_ascii() {
		let mut md = MetadataMap::new();
		md.append("yes-bib", AsciiMetadataValue::from_static("fail"));
	}

	#[test]
	fn test_interceptor_unauthorized_request() {
		let mut interceptor = JwtAuthInterceptor::new("broken key".into());
		let request = Request::new(());
		let result = interceptor.call(request);

		match result {
			Err(s) => assert!(s.code() == Code::PermissionDenied),
			Ok(_) => panic!("An unauthorized request slipped through interceptor"),
		}
	}

	#[test]
	fn test_interceptor_happy_request() {
		let request = happy_request();
		let mut interceptor = JwtAuthInterceptor::new(JWT_PUBLIC_KEY.into());
		interceptor.call(request).expect("Happy request failed authorization");
	}

	#[test]
	fn test_interceptor_request_with_bad_user_is_corrected() {
		let mut request = happy_request();
		let md = request.metadata_mut();
		md.insert("user", "penguins".parse().unwrap());
		let mut interceptor = JwtAuthInterceptor::new(JWT_PUBLIC_KEY.into());

		let modified_request = interceptor.call(request).expect("Interceptor failed to process happy request");
		let md = modified_request.metadata();

		let uuid = md.get("user").unwrap();
		assert!(uuid == JWT_TOKEN_EXTRACTED_UUID);
	}

	fn happy_request() -> Request<()> {
		let mut request = Request::new(());
		let md = request.metadata_mut();
		let auth_value = format!("JWT {}", JWT_TOKEN).parse().unwrap();
		md.insert(AUTH_METADATA_KEY, auth_value);

		request
	}

	const JWT_TOKEN_EXTRACTED_UUID: &str = "deadbeefbeefbeefbeefdeadbeef6666";

	const AUTH_METADATA_KEY: &str = "authorization";

	const JWT_TOKEN: &str = "eyJleHAiOjk5OTk5OTk5OTk5LCJ1c2VyIjoiZGVhZGJlZWYtYm\
VlZi1iZWVmLWJlZWYtZGVhZGJlZWY2NjY2In0.5TM5QO1qYU5Xe2RkwC_V_pr7UXTgr7BCkKrKUByfM\
-EvRd1bbwlxydyhPikeQ5iNwPG_d6XmqiSqw-xt3YG_9Q";

	const JWT_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEVVHNXKxoUNkoX9hnOJpSz6K2KDfi
gxaSXu+FIpP32qvcDgZPZ01tjnGjOysyPxRoZaMu/d9rHi3ulbceoYwS+Q==
-----END PUBLIC KEY-----";
}
