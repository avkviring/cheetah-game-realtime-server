use crate::grpc::userstore::{
	get_double_reply, get_long_reply, get_string_reply, GetDoubleReply, GetLongReply,
	GetStringReply, Status,
};

impl From<i64> for GetLongReply {
	fn from(v: i64) -> Self {
		Self {
			result: Some(get_long_reply::Result::Value(v)),
		}
	}
}

impl From<Status> for GetLongReply {
	fn from(s: Status) -> Self {
		Self {
			result: Some(get_long_reply::Result::Status(s as i32)),
		}
	}
}

impl From<f64> for GetDoubleReply {
	fn from(v: f64) -> Self {
		Self {
			result: Some(get_double_reply::Result::Value(v)),
		}
	}
}

impl From<Status> for GetDoubleReply {
	fn from(s: Status) -> Self {
		Self {
			result: Some(get_double_reply::Result::Status(s as i32)),
		}
	}
}

impl From<String> for GetStringReply {
	fn from(s: String) -> Self {
		Self {
			result: Some(get_string_reply::Result::Value(s)),
		}
	}
}

impl From<Status> for GetStringReply {
	fn from(s: Status) -> Self {
		Self {
			result: Some(get_string_reply::Result::Status(s as i32)),
		}
	}
}
