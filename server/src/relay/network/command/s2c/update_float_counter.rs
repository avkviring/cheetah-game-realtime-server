use crate::relay::network::command::s2c::S2CCommand;
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::objects::object::FieldID;
use crate::relay::room::room::GlobalObjectId;

#[derive(Debug, PartialEq, Clone)]
pub struct UpdateFloatCounterS2CCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub value: f64,
}

impl S2CCommand for UpdateFloatCounterS2CCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> bool {
		buffer
			.write_u64(self.global_object_id)
			.and_then(|_| buffer.write_u16(self.field_id))
			.and_then(|_| buffer.write_f64(self.value))
			.is_ok()
	}
}