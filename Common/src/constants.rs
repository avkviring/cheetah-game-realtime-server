pub type FieldID = u16;
pub type GroupType = u64;
pub type ClientId = u16;
pub type LocalObjectId = u32;
pub type GlobalObjectId = u64;

///
/// Максимальное количество FieldId в игровом объекте (для каждого типа данных)
///
pub const MAX_FIELDS_IN_OBJECT: usize = 256;

///
/// Максимальный размер поля в struct/event
///
pub const MAX_SIZE_STRUCT: usize = 255;
