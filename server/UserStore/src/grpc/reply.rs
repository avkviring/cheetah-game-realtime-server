use crate::grpc::userstore::{fetch_reply, FetchReply, PrimitiveValue, Status};

impl From<Status> for FetchReply {
	fn from(s: Status) -> Self {
		Self {
			result: Some(fetch_reply::Result::Status(s as i32)),
		}
	}
}

impl From<PrimitiveValue> for FetchReply {
	fn from(v: PrimitiveValue) -> Self {
		Self {
			result: Some(fetch_reply::Result::Value(v)),
		}
	}
}
