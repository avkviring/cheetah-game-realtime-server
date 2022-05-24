use cheetah_matches_relay_common::{
	commands::{
		s2c::S2CCommand,
		types::{float::SetDoubleCommand, long::SetLongCommand, structure::SetStructureCommand},
		FieldType,
	},
	room::object::GameObjectId,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
	Long(i64),
	Double(f64),
	Structure(Vec<u8>),
}

impl FieldValue {
	pub fn get_type(&self) -> FieldType {
		match self {
			FieldValue::Long(_) => FieldType::Long,
			FieldValue::Double(_) => FieldType::Double,
			FieldValue::Structure(_) => FieldType::Structure,
		}
	}

	pub fn s2c_set_command(&self, object_id: GameObjectId, field_id: u16) -> S2CCommand {
		match self {
			FieldValue::Long(value) => S2CCommand::SetLong(SetLongCommand {
				field_id,
				object_id,
				value: *value,
			}),
			FieldValue::Double(value) => S2CCommand::SetDouble(SetDoubleCommand {
				object_id,
				field_id,
				value: *value,
			}),
			FieldValue::Structure(value) => S2CCommand::SetStructure(SetStructureCommand {
				object_id,
				field_id,
				value: value.as_slice().into(),
            }),
		}
	}
}
