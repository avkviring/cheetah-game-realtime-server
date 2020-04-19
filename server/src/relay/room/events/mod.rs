use bytebuffer::ByteBuffer;

use crate::relay::room::clients::Client;

pub mod create_game_object;
pub mod delete_game_object;
pub mod update_long_counter;
pub mod update_float_counter;
pub mod event;
pub mod update_struct;


/// события одного игрового цикла
/// накапливаем изменения
/// и когда настанет время - отправляем их клиентам
pub struct S2CCommandCollector {
	commands: Vec<Box<dyn S2CCommand>>
}

pub struct AffectedClients {}

pub trait S2CCommand {
	/// получить идентификатор команды
	fn get_command_id(&self) -> u8;
	
	/// список затронутых клиентов
	fn get_affected_clients(&self) -> &AffectedClients;
	
	/// преобразовать команду в поток байт
	fn encode(&self, bytes: &mut ByteBuffer);
}

impl S2CCommandCollector {}


impl Default for S2CCommandCollector {
	fn default() -> Self {
		return S2CCommandCollector {
			commands: Default::default()
		};
	}
}