pub type GameObjectTemplateId = u16;
///
/// Максимальное количество `FieldId` в игровом объекте (для каждого типа данных)
///
pub const MAX_FIELDS_IN_OBJECT: usize = 255;

///
/// Максимальный размер поля в struct/event
///
pub const MAX_SIZE_STRUCT: usize = 255;

pub const ALL_STRUCTURES_SIZE: usize = MAX_FIELDS_IN_OBJECT * MAX_SIZE_STRUCT;
