use crate::ydb::table::ToDbTable;
use cheetah_libraries_ydb::converters::YDBValueConverter;

pub trait PrimitiveValue: Sized + Clone + YDBValueConverter + ToDbTable {}

impl PrimitiveValue for i64 {}
impl PrimitiveValue for f64 {}
impl PrimitiveValue for &str {}
impl PrimitiveValue for String {}
