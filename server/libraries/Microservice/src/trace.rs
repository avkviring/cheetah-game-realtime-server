use std::fmt::Debug;

use tonic::Status;
use tracing::error;

///
/// Использовать для внешних запросов, не раскрываем информацию об ошибке клиенту
///
#[track_caller]
pub fn trace_and_convert_to_tonic_internal_status<T>(error: T) -> Status
where
	T: Debug,
{
	trace(error);
	Status::internal("internal error")
}

///
/// Использовать для внешних запросов, не раскрываем информацию об ошибке клиенту
///
#[track_caller]
pub fn trace_and_convert_to_tonic_unauthenticated_status<T>(error: T) -> Status
where
	T: Debug,
{
	trace(error);
	Status::unauthenticated("unauthenticated error")
}

///
/// Использовать для  внутренних запросов, раскрываем информацию об ошибке клиенту
///
#[track_caller]
pub fn trace_and_convert_to_tonic_internal_status_with_full_message<T>(error: T) -> Status
where
	T: Debug,
{
	Status::internal(trace(error))
}

///
/// Использовать для  внутренних запросов, раскрываем информацию об ошибке клиенту
///
#[track_caller]
pub fn trace_and_convert_to_tonic_unauthenticated_status_with_full_message<T>(error: T) -> Status
where
	T: Debug,
{
	Status::unauthenticated(trace(error))
}

fn trace<T>(error: T) -> String
where
	T: Debug,
{
	let msg = format!("{} {:?}", std::panic::Location::caller(), error);
	error!("{}", msg);
	msg
}
