use crate::room::object::GameObjectId;

///
/// удаление игрового объекта
/// - направления C->S, S->C
///
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(C)]
pub struct DeleteGameObjectCommand {
	pub object_id: GameObjectId,
}
