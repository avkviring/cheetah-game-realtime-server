use cheetah_common::commands::binary_value::Buffer;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::structure::SetStructureCommand;
use cheetah_common::room::object::GameObjectId;

use crate::room::object::fields::FieldValue;

impl FieldValue for Buffer {
	fn into(&self, object_id: GameObjectId, field_id: FieldId) -> S2CCommand {
		S2CCommand::SetStructure(SetStructureCommand {
			object_id,
			field_id,
			value: self.clone(),
		})
	}
}