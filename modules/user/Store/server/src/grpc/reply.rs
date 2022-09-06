use crate::grpc::userstore::{
	fetch_double_reply, fetch_long_reply, fetch_string_reply, FetchDoubleReply, FetchLongReply, FetchStatus, FetchStringReply,
};

impl From<FetchStatus> for FetchDoubleReply {
	fn from(s: FetchStatus) -> Self {
		Self {
			result: Some(fetch_double_reply::Result::Status(s as i32)),
		}
	}
}

impl From<FetchStatus> for FetchLongReply {
	fn from(s: FetchStatus) -> Self {
		Self {
			result: Some(fetch_long_reply::Result::Status(s as i32)),
		}
	}
}

impl From<FetchStatus> for FetchStringReply {
	fn from(s: FetchStatus) -> Self {
		Self {
			result: Some(fetch_string_reply::Result::Status(s as i32)),
		}
	}
}
