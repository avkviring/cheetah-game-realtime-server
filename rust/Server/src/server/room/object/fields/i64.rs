use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::long::SetLongCommand;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::server::room::object::fields::FieldValue;

impl FieldValue for i64 {
	fn into(&self, object_id: GameObjectId, field_id: FieldId) -> S2CCommand {
		S2CCommand::SetLong(SetLongCommand { object_id, field_id, value: *self })
	}
}
