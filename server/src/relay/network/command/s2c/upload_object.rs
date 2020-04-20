use bytebuffer::ByteBuffer;

use crate::relay::network::command::s2c::{AffectedClients, S2CCommand};
use crate::relay::room::objects::object::GameObject;
use crate::relay::room::room::GlobalObjectId;

/// Загрузка объекта на клиент
/// со всеми данными
pub struct UploadObjectS2CCommand {
	pub affected_clients: AffectedClients,
	pub cloned_object: GameObject,
}

impl S2CCommand for UploadObjectS2CCommand {
	fn get_command_id(&self) -> u8 {
		7
	}
	
	fn get_affected_clients(&self) -> &AffectedClients {
		return &self.affected_clients;
	}
	
	fn encode(&self, bytes: &mut ByteBuffer) {
		bytes.write_u64(self.cloned_object.id);
		let structures = &self.cloned_object.structures;
		bytes.write_u16(structures.len() as u16);
		for (id, data) in structures {
			bytes.write_u16(*id);
			bytes.write_u16(data.data.len() as u16);
			bytes.write_bytes(&*data.data)
		}
		
		let long_counters = &self.cloned_object.long_counters;
		bytes.write_u16(long_counters.len() as u16);
		for (id, data) in long_counters {
			bytes.write_u16(*id);
			bytes.write_i64(data.counter);
		}
		
		let float_counters = &self.cloned_object.float_counters;
		bytes.write_u16(float_counters.len() as u16);
		for (id, data) in float_counters {
			bytes.write_u16(*id);
			bytes.write_f64(data.counter);
		}
	}
}
