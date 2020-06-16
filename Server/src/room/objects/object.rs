use cheetah_relay_common::constants::{ClientId, FieldID};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::room::listener::RoomListener;
use crate::room::Room;
use crate::room::objects::id::ServerGameObjectId;


///
/// Игровой объект
/// содержит данные от пользователей
///
#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
	pub id: ServerGameObjectId,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
}


#[derive(Debug)]
pub enum ObjectFieldType {
	LongCounter,
	FloatCounter,
	Struct,
	Event,
}


impl GameObject {
	pub fn new(id: ServerGameObjectId, access_groups: AccessGroups, fields: GameObjectFields) -> GameObject {
		GameObject {
			id,
			access_groups,
			fields,
		}
	}
	pub fn get_long_counter(&self, field_id: FieldID) -> i64 {
		*self.fields.long_counters.get(&field_id).unwrap_or(&0)
	}
	pub fn get_float_counter(&self, field_id: FieldID) -> f64 {
		*self.fields.float_counters.get(&field_id).unwrap_or(&0.0)
	}
	pub fn get_struct(&self, field_id: FieldID) -> Option<&Vec<u8>> {
		self.fields.structures.get(&field_id)
	}
}


impl Room {
	pub fn object_increment_long_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: i64) -> i64 {
		let result = value + object
			.fields
			.long_counters
			.get(&field_id).unwrap_or(&Default::default());
		
		object.fields.long_counters.insert(field_id, result);
		self.listener.on_object_long_counter_change(field_id, object, &self.clients);
		result
	}
	
	pub fn object_set_long_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: i64) {
		object.fields.long_counters.insert(field_id, value);
		self.listener.on_object_long_counter_set(field_id, object, &self.clients);
	}
	
	pub fn object_increment_float_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: f64) -> f64 {
		let result = value + object
			.fields
			.float_counters
			.get(&field_id).unwrap_or(&Default::default());
		object.fields.float_counters.insert(field_id, result);
		self.listener.on_object_float_counter_change(field_id, object, &self.clients);
		result
	}
	
	pub fn object_set_float_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: f64) {
		object.fields.float_counters.insert(field_id, value);
		self.listener.on_object_float_counter_set(field_id, object, &self.clients);
	}
	
	pub fn object_update_struct(&mut self, object: &mut GameObject, field_id: FieldID, value: Vec<u8>) {
		object.fields.structures.insert(field_id, value);
		self.listener.on_object_struct_updated(field_id, object, &self.clients);
	}
	
	pub fn object_send_event(&mut self, object: &mut GameObject, field_id: FieldID, event_data: &Vec<u8>) {
		self.listener.on_object_event_fired(field_id, event_data, object, &self.clients);
	}
}


