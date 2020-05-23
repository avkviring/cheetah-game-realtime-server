use crate::network::command::s2c::S2CCommand;
use crate::network::types::niobuffer::NioBuffer;
use crate::room::objects::object::FieldID;
use crate::room::room::GlobalObjectId;

#[derive(Debug, PartialEq, Clone)]
pub struct UpdateLongCounterS2CCommand {
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub value: i64,
}

impl S2CCommand for UpdateLongCounterS2CCommand {
	fn encode(&self, bytes: &mut NioBuffer) -> bool {
		bytes
			.write_u64(self.global_object_id)
			.and_then(|_| bytes.write_u16(self.field_id))
			.and_then(|_| bytes.write_i64(self.value))
			.is_ok()
	}
}
