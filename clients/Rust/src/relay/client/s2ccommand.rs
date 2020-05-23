#[repr(C)]
pub struct UpdateLongCounterS2C {
	object_id: u64,
	field_id: u16,
	value: i64,
}


pub enum S2CCommandUnion {
	UpdateLongCounter(UpdateLongCounterS2C)
}
