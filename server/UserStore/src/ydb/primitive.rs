use crate::ydb::table::ToDbTable;
use cheetah_libraries_ydb::converters::YDBValueConverter;

pub trait Primitive: Sized + Clone + YDBValueConverter + ToDbTable {}

impl Primitive for i64 {}
impl Primitive for f64 {}
impl Primitive for &str {}
impl Primitive for String {}
