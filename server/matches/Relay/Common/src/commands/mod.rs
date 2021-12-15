pub mod c2s;
pub mod s2c;
pub mod types;

pub type CommandBuffer = heapless::Vec<u8, 256>;

///
/// Идентификатор типа команды
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CommandTypeId(pub u8);

impl CommandTypeId {
	const CREATE: CommandTypeId = CommandTypeId(0);
	const CREATED: CommandTypeId = CommandTypeId(1);
	const SET_LONG: CommandTypeId = CommandTypeId(2);
	const INCREMENT_LONG: CommandTypeId = CommandTypeId(3);
	const COMPARE_AND_SET_LONG: CommandTypeId = CommandTypeId(4);
	const SET_DOUBLE: CommandTypeId = CommandTypeId(5);
	const INCREMENT_DOUBLE: CommandTypeId = CommandTypeId(6);
	const SET_STRUCTURE: CommandTypeId = CommandTypeId(7);
	const EVENT: CommandTypeId = CommandTypeId(8);
	const TARGET_EVENT: CommandTypeId = CommandTypeId(9);
	const DELETE: CommandTypeId = CommandTypeId(10);
	const ATTACH_TO_ROOM: CommandTypeId = CommandTypeId(11);
	const DETACH_FROM_ROOM: CommandTypeId = CommandTypeId(12);
}

///
/// Тип данных поля
///
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum FieldType {
	Long,
	Double,
	Structure,
	Event,
}
