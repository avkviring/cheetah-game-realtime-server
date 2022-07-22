use crate::grpc::userstore::{
	fetch_double_reply, fetch_long_reply, fetch_string_reply, FetchDoubleReply, FetchLongReply,
	FetchStringReply, Status,
};

impl From<Status> for FetchDoubleReply {
	fn from(s: Status) -> Self {
		Self {
			result: Some(fetch_double_reply::Result::Status(s as i32)),
		}
	}
}

impl From<Status> for FetchLongReply {
	fn from(s: Status) -> Self {
		Self {
			result: Some(fetch_long_reply::Result::Status(s as i32)),
		}
	}
}

impl From<Status> for FetchStringReply {
	fn from(s: Status) -> Self {
		Self {
			result: Some(fetch_string_reply::Result::Status(s as i32)),
		}
	}
}
