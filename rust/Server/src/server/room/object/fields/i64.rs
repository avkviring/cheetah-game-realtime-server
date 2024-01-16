use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::long::LongField;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::server::room::object::fields::FieldValue;

impl FieldValue for i64 {
	fn into(&self, object_id: GameObjectId, field_id: FieldId, collector: &mut Vec<S2CCommand>) {
		collector.push(S2CCommand::SetLong(LongField { object_id, field_id, value: *self }));
	}
}
