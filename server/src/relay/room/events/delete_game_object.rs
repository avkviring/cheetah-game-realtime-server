use bytebuffer::ByteBuffer;

use crate::relay::room::events::{AffectedClients, S2CCommand};
use crate::relay::room::room::GlobalObjectId;

struct DeleteObjectS2CCommand {
	affected_clients: AffectedClients,
	global_object_id: GlobalObjectId,
}

impl S2CCommand for DeleteObjectS2CCommand {
	fn get_command_id(&self) -> u8 {
		2
	}
	
	fn get_affected_clients(&self) -> &AffectedClients {
		return &self.affected_clients;
	}
	
	fn encode(&self, bytes: &mut ByteBuffer) {
		bytes.write_u64(self.global_object_id)
	}
}
