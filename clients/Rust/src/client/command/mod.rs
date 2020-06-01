use crate::client::command::event::{ReceiveEventS2C, SendEventC2S};
use crate::client::command::float_counter::{IncrementFloatCounterC2S, SetFloatCounterS2C};
use crate::client::command::long_counter::{IncrementLongCounterC2S, SetLongCounterS2C};
use crate::client::command::structure::{SetStructC2S, SetStructS2C};
use crate::client::command::unload::{UnloadObjectC2S, UnloadObjectS2C};
use crate::client::command::upload::{UploadObjectC2S, UploadObjectS2C};

pub mod upload;
pub mod long_counter;
pub mod float_counter;
pub mod structure;
pub mod event;
pub mod unload;

pub enum C2SCommandUnion {
	Upload(UploadObjectC2S),
	IncrementLongCounter(IncrementLongCounterC2S),
	IncrementFloatCounter(IncrementFloatCounterC2S),
	SetStruct(SetStructC2S),
	SendEvent(SendEventC2S),
	Unload(UnloadObjectC2S),
}

pub enum S2CCommandUnion {
	Upload(UploadObjectS2C),
	SetLongCounter(SetLongCounterS2C),
	SetFloatCounter(SetFloatCounterS2C),
	SetStruct(SetStructS2C),
	ReceiveEvent(ReceiveEventS2C),
	Unload(UnloadObjectS2C),
}
