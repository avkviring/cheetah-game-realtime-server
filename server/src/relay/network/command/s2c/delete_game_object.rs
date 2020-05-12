use crate::relay::network::command::s2c::S2CCommand;
use crate::relay::network::types::niobuffer::NioBuffer;
use crate::relay::room::room::GlobalObjectId;

#[derive(Debug, PartialEq, Clone)]
pub struct DeleteGameObjectS2CCommand {
	pub global_object_id: GlobalObjectId,
}

impl S2CCommand for DeleteGameObjectS2CCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> bool {
		buffer.write_u64(self.global_object_id).is_ok()
	}
}
