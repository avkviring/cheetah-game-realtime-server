use crate::{
	grpc::userstore::{GetDoubleRequest, GetLongRequest, GetStringRequest},
	ydb::primitive::Primitive,
};

use super::userstore::SetLongRequest;

pub trait RequestWithField {
	fn field_name(&self) -> &str;
}

impl RequestWithField for GetDoubleRequest {
	fn field_name(&self) -> &str {
		&self.field_name
	}
}

impl RequestWithField for GetLongRequest {
	fn field_name(&self) -> &str {
		&self.field_name
	}
}

impl RequestWithField for GetStringRequest {
	fn field_name(&self) -> &str {
		&self.field_name
	}
}

pub trait RequestWithValue<T: Primitive> {
	fn value(&self) -> &T;
}

impl RequestWithValue<i64> for SetLongRequest {
	fn value(&self) -> &i64 {
		&self.value
	}
}
