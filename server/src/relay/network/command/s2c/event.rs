use crate::relay::network::command::s2c::S2CCommand;
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::objects::object::FieldID;
use crate::relay::room::room::GlobalObjectId;

#[derive(Debug, PartialEq, Clone)]
pub struct EventS2CCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub event: Vec<u8>,
}

impl S2CCommand for EventS2CCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> bool {
		buffer
			.write_u64(self.global_object_id)
			.and_then(|_| buffer.write_u16(self.field_id))
			.and_then(|_| buffer.write_u16(self.event.len() as u16))
			.and_then(|_| buffer.write_bytes(&self.event))
			.is_ok()
	}
}
