pub trait Primitive: Sized + Clone {}

impl Primitive for i64 {}
impl Primitive for f64 {}
impl Primitive for &str {}
