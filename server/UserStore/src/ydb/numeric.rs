use std::ops::Add;

use crate::ydb::primitive::PrimitiveValue;

pub trait Num: PrimitiveValue + Add {}

impl Num for i64 {}
impl Num for f64 {}
