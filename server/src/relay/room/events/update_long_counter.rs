use bytebuffer::ByteBuffer;

use crate::relay::room::events::{AffectedClients, S2CCommand};
use crate::relay::room::objects::object::FieldID;
use crate::relay::room::room::GlobalObjectId;

struct UpdateLongCounterS2CCommand {
	affected_clients: AffectedClients,
	global_object_id: GlobalObjectId,
	field_id: FieldID,
	value: i64,
}

impl S2CCommand for UpdateLongCounterS2CCommand {
	fn get_command_id(&self) -> u8 {
		3
	}
	
	fn get_affected_clients(&self) -> &AffectedClients {
		return &self.affected_clients;
	}
	
	fn encode(&self, bytes: &mut ByteBuffer) {
		bytes.write_u64(self.global_object_id);
		bytes.write_u16(self.field_id);
		bytes.write_i64(self.value);
	}
}
