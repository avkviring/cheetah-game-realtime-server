use std::fmt::Debug;

use tonic::Status;
use tracing::error;

#[track_caller]
pub fn trace_error_and_convert_to_internal_tonic_status<T>(t: T) -> Status
where
	T: Debug,
{
	error!("{} {:?}", std::panic::Location::caller(), t);
	Status::internal("internal error")
}

#[track_caller]
pub fn trace_error_and_convert_to_unauthenticated_tonic_status<T>(t: T) -> Status
where
	T: Debug,
{
	error!("{} {:?}", std::panic::Location::caller(), t);
	Status::unauthenticated("unauthenticated error")
}
