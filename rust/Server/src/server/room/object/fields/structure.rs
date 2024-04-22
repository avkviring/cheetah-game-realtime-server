use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::structure::BinaryField;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::server::room::object::fields::FieldValue;

pub type Structure = Buffer;
impl FieldValue for Box<Structure> {
	fn into(&self, object_id: GameObjectId, field_id: FieldId, collector: &mut Vec<S2CCommand>) {
		collector.push(S2CCommand::SetStructure(
			BinaryField {
				object_id,
				field_id,
				value: *self.clone(),
			}
			.into(),
		));
	}
}
