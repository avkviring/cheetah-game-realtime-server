use std::fmt::Debug;

use tracing::error;

pub trait Trace<T> {
	fn trace_err(self, details: impl Debug) -> Result<T, String>;
}

impl<T, E: Debug> Trace<T> for Result<T, E> {
	fn trace_err(self, details: impl Debug) -> Result<T, String> {
		match self {
			Ok(v) => Ok(v),
			Err(e) => Err(trace_err(details, e)),
		}
	}
}

pub fn trace_err(details: impl Debug, object: impl Debug) -> String {
	let msg = format!("{:?} {:?}", details, object);
	error!("{}", msg);
	msg
}
