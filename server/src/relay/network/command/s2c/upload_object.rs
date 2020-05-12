use std::collections::HashMap;

use crate::relay::network::command::s2c::S2CCommand;
use crate::relay::network::types::niobuffer::{NioBuffer, NioBufferError};
use crate::relay::room::objects::object::{DataStruct, FieldID, FloatCounter, GameObject, LongCounter};

/// Загрузка объекта на клиент
/// со всеми данными
#[derive(Debug, PartialEq, Clone)]
pub struct UploadGameObjectS2CCommand {
	pub cloned_object: GameObject,
}

impl S2CCommand for UploadGameObjectS2CCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> bool {
		let structures = &self.cloned_object.structures;
		let long_counters = &self.cloned_object.long_counters;
		let float_counters = &self.cloned_object.float_counters;
		
		buffer
			.write_u64(self.cloned_object.id)
			.and_then(|_| buffer.write_u16(long_counters.len() as u16))
			.and_then(|_| buffer.write_u16(float_counters.len() as u16))
			.and_then(|_| buffer.write_u16(structures.len() as u16))
			.and_then(|_| write_long_counters(buffer, long_counters))
			.and_then(|_| write_float_counters(buffer, float_counters))
			.and_then(|_| write_structures(buffer, structures))
			.is_ok()
	}
}


fn write_long_counters(buffer: &mut NioBuffer, counters: &HashMap<FieldID, LongCounter>) -> Result<(), NioBufferError> {
	counters.iter().map(|(id, data)|
		{
			buffer
				.write_u16(*id)
				.and_then(|_| buffer.write_i64(data.counter))
		})
		.find(|f| f.is_err())
		.unwrap_or_else(|| Result::Ok(()))
}

fn write_float_counters(buffer: &mut NioBuffer, counters: &HashMap<FieldID, FloatCounter>) -> Result<(), NioBufferError> {
	counters.iter().map(|(id, data)|
		{
			buffer
				.write_u16(*id)
				.and_then(|_| buffer.write_f64(data.counter))
		})
		.find(|f| f.is_err())
		.unwrap_or_else(|| Result::Ok(()))
}

fn write_structures(buffer: &mut NioBuffer, structures: &HashMap<FieldID, DataStruct>) -> Result<(), NioBufferError> {
	structures.iter().map(|(id, data)|
		{
			buffer
				.write_u16(*id)
				.and_then(|_| buffer.write_u16(data.data.len() as u16))
				.and_then(|_| buffer.write_bytes(&*data.data))
		})
		.find(|f| f.is_err())
		.unwrap_or_else(|| Result::Ok(()))
}

