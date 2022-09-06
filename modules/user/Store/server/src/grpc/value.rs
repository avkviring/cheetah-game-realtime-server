use crate::grpc::userstore::{fetch_double_reply, fetch_long_reply, fetch_string_reply, FetchDoubleReply, FetchLongReply, FetchStringReply};

impl From<f64> for FetchDoubleReply {
	fn from(v: f64) -> Self {
		Self {
			result: Some(fetch_double_reply::Result::Value(v)),
		}
	}
}

impl From<i64> for FetchLongReply {
	fn from(v: i64) -> Self {
		Self {
			result: Some(fetch_long_reply::Result::Value(v)),
		}
	}
}

impl From<String> for FetchStringReply {
	fn from(s: String) -> Self {
		Self {
			result: Some(fetch_string_reply::Result::Value(s)),
		}
	}
}
