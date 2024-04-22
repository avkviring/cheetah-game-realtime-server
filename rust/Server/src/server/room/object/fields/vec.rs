use std::collections::VecDeque;

use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::structure::BinaryField;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::server::room::object::fields::structure::Structure;
use crate::server::room::object::fields::FieldValue;

pub type Items = VecDeque<Box<Structure>>;

impl FieldValue for Items {
	fn into(&self, object_id: GameObjectId, field_id: FieldId, collector: &mut Vec<S2CCommand>) {
		self.iter()
			.map(|item| S2CCommand::AddItem(BinaryField { object_id, field_id, value: **item }.into()))
			.for_each(|command| collector.push(command))
	}
}
