use bytebuffer::ByteBuffer;

use crate::relay::network::command::s2c::{AffectedClients, S2CCommand};
use crate::relay::room::objects::object::FieldID;
use crate::relay::room::room::GlobalObjectId;

pub struct EventS2CCommand {
	pub affected_clients: AffectedClients,
	pub global_object_id: GlobalObjectId,
	pub field_id: FieldID,
	pub event: Vec<u8>,
}

impl S2CCommand for EventS2CCommand {
	fn get_command_id(&self) -> u8 {
		6
	}
	
	fn get_affected_clients(&self) -> &AffectedClients {
		return &self.affected_clients;
	}
	
	fn encode(&self, bytes: &mut ByteBuffer) {
		bytes.write_u64(self.global_object_id);
		bytes.write_u16(self.field_id);
		bytes.write_u16(self.event.len() as u16);
		bytes.write_bytes(&self.event)
	}
}
