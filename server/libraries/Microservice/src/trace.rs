use std::fmt::Debug;

use tracing::error;

pub trait ResultErrorTracer<T, E> {
	fn trace_and_map_msg<M, O, F>(self, details: M, op: O) -> Result<T, F>
	where
		O: FnOnce(String) -> F,
		M: Debug;
}

impl<T, E> ResultErrorTracer<T, E> for Result<T, E>
where
	E: Debug,
{
	fn trace_and_map_msg<M, O, F>(self, details: M, op: O) -> Result<T, F>
	where
		O: FnOnce(String) -> F,
		M: Debug,
	{
		match self {
			Ok(v) => Ok(v),
			Err(e) => {
				let msg = trace(&details, &e);
				Err(op(msg))
			}
		}
	}
}

pub fn trace<M: Debug, E: Debug>(details: &M, error: &E) -> String {
	let msg = format!("{:?} {:?}", details, error);
	error!("{}", msg);
	msg
}
