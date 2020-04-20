use bytebuffer::ByteBuffer;
use crate::relay::room::room::GlobalObjectId;
use crate::relay::network::command::s2c::{AffectedClients, S2CCommand};

pub struct CreateObjectS2CCommand {
	pub(crate) affected_clients: AffectedClients,
	pub(crate) global_object_id: GlobalObjectId,
}

impl S2CCommand for CreateObjectS2CCommand {
	fn get_command_id(&self) -> u8 {
		1
	}
	
	fn get_affected_clients(&self) -> &AffectedClients {
		return &self.affected_clients;
	}
	
	fn encode(&self, bytes: &mut ByteBuffer) {
		bytes.write_u64(self.global_object_id)
	}
}
