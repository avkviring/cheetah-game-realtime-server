use std::ops::Add;

use crate::ydb::primitive::Primitive;

pub trait Num: Primitive + Add {}

impl Num for i64 {}
impl Num for f64 {}
