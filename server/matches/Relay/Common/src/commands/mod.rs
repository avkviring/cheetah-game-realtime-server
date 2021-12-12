pub mod c2s;
pub mod s2c;
pub mod types;

pub type HeaplessBuffer = heapless::Vec<u8, 256>;

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
