pub mod c2s;
pub mod s2c;
pub mod types;

pub type CommandBuffer = heapless::Vec<u8, 256>;

///
/// Идентификатор типа команды
///
#[derive(Debug, Copy, Clone)]
pub struct CommandTypeId(pub u8);

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
