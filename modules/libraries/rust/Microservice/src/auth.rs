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
	use tonic::metadata::MetadataValue;

	use super::{
		store_user_uuid, AsciiMetadataValue, Interceptor, JwtAuthInterceptor, MetadataMap, Request,
		Uuid,
	};

	#[test]
	fn test_store_user_uuid() {
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
	#[should_panic]
	fn test_interceptor_unauthorized_request() {
		let mut interceptor = JwtAuthInterceptor::new("lol".into());
		let request = Request::new(());
		interceptor.call(request).expect("Bad request metadata");
	}

	#[test]
	fn test_interceptor_happy_request() {
		let mut request = Request::new(());
		let md = request.metadata_mut();
		let auth_value = MetadataValue::from_str(&format!("Bear {}", JWT_TOKEN)).unwrap();
		md.insert(AUTH_METADATA_KEY, auth_value);
		let mut interceptor = JwtAuthInterceptor::new(JWT_PUBLIC_KEY.into());
		interceptor
			.call(request)
			.expect("Happy request failed authorization");
	}

	const AUTH_METADATA_KEY: &str = "authorization";

	const JWT_TOKEN: &str = "eyJhbGciOiJFUzI1NiJ9.eyJuYW1lIjoiQXBwbGUifQ.sTBwTCkNT3eBeFi6vuxZVIHBFTOqdUeodl-xKPitRMXeIcqe31CT-cQ1m2WFLGGeCLRfsjZSsdbELU-8gXSBJg";

	const JWT_PUBLIC_KEY: &str = "----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEVVHNXKxoUNkoX9hnOJpSz6K2KDfi
gxaSXu+FIpP32qvcDgZPZ01tjnGjOysyPxRoZaMu/d9rHi3ulbceoYwS+Q==
-----END PUBLIC KEY-----";

	const JWT_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgcg7dsJWSz8f40cEv
BTeGSzANXGlEzutd9IIm6/inl0ahRANCAARVUc1crGhQ2Shf2Gc4mlLPorYoN+KD
FpJe74Uik/faq9wOBk9nTW2OcaM7KzI/FGhloy7932seLe6Vtx6hjBL5
-----END PRIVATE KEY-----";
}
