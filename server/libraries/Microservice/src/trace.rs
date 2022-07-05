use std::fmt::Debug;

use tracing::error;

pub trait ResultErrorTracer<T, E> {
	fn trace_and_map_err<M, F, OutError>(self, details: M, f: F) -> Result<T, OutError>
	where
		F: FnOnce(E) -> OutError,
		M: Into<String>;
}

impl<T, E> ResultErrorTracer<T, E> for Result<T, E>
where
	E: Debug,
{
	fn trace_and_map_err<M, F, OutError>(self, details: M, f: F) -> Result<T, OutError>
	where
		F: FnOnce(E) -> OutError,
		M: Into<String>,
	{
		match self {
			Ok(v) => Ok(v),
			Err(e) => {
				let msg = format!("{} {:?}", details.into(), e);
				error!("{}", msg);
				Err(f(e))
			}
		}
	}
}
