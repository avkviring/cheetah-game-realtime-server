use ydb::{YdbError, YdbOrCustomerError};
use ydb_grpc::ydb_proto::status_ids::StatusCode;

pub fn is_unique_violation_error(result: &YdbOrCustomerError) -> bool {
	matches!(result, 
		YdbOrCustomerError::YDB(YdbError::YdbStatusError(status)) 
		if status.operation_status().unwrap() == StatusCode::PreconditionFailed)
}
