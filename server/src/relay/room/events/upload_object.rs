use bytebuffer::ByteBuffer;

use crate::relay::room::events::{AffectedClients, S2CCommand};
use crate::relay::room::objects::object::GameObject;
use crate::relay::room::room::GlobalObjectId;

/// Загрузка объекта на клиент
/// со всеми данными
struct UploadObjectS2CCommand {
	affected_clients: AffectedClients,
	cloned_object: GameObject,
}

impl S2CCommand for UploadObjectS2CCommand {
	fn get_command_id(&self) -> u8 {
		7
	}
	
	fn get_affected_clients(&self) -> &AffectedClients {
		return &self.affected_clients;
	}
	
	fn encode(&self, bytes: &mut ByteBuffer) {
		bytes.write_u64(self.cloned_object.global_object_id);
		let structures = &self.cloned_object.structures;
		bytes.write_u16(structures.len() as u16);
		for (id, data) in structures {
			bytes.write_u16(*id);
			bytes.write_u16(*data.data.len());
			bytes.write_bytes(&*data.data)
		}
		
		let long_counters = &self.cloned_object.long_counters;
		bytes.write_u16(long_counters.len() as u16);
		for (id, data) in long_counters {
			bytes.write_u16(*id);
			bytes.write_i64(*data.counter);
		}
		
		let float_counters = &self.cloned_object.float_counters;
		bytes.write_u16(float_counters.len() as u16);
		for (id, data) in float_counters {
			bytes.write_u16(*id);
			bytes.write_i64(*data.counter);
		}
	}
}
