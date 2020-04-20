use std::collections::HashMap;
use std::ops::Shl;

use log::Level::Debug;

use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::listener::RoomListener;
use crate::relay::room::objects::owner::Owner;
use crate::relay::room::room::Room;

pub type FieldID = u16;

/// Игровой объект
/// содержит данные от пользователей
pub struct GameObject {
	pub id: u64,
	pub owner: Owner,
	/// счетчики
	pub long_counters: HashMap<FieldID, LongCounter>,
	pub float_counters: HashMap<FieldID, FloatCounter>,
	/// структуры (для сервера это массивы данных)
	pub structures: HashMap<FieldID, DataStruct>,
	/// группы доступа
	pub groups: AccessGroups,
}


/// счетчик
pub struct LongCounter {
	pub counter: i64
}

/// счетчик
pub struct FloatCounter {
	pub counter: f64
}

/// данные
pub struct DataStruct {
	pub data: Vec<u8>
}

#[derive(Debug)]
pub enum ObjectFieldType {
	LongCounter,
	FloatCounter,
	Struct,
	StringToIdMap,
	IdSet,
	Event,
}


impl GameObject {
	pub fn new_client_object(client: &Client, local_object_id: u32, groups: AccessGroups) -> GameObject {
		GameObject {
			id: GameObject::to_global_object_id(client, local_object_id),
			owner: Owner::new_owner(client),
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			groups,
		}
	}
	
	pub fn new_root_object(id: u64, groups: AccessGroups) -> GameObject {
		GameObject {
			id,
			owner: Owner::new_root_owner(),
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			groups,
		}
	}
	
	pub fn update_struct(&mut self, field_id: FieldID, data: Vec<u8>) {
		self.structures.insert(field_id, DataStruct { data });
	}
	
	pub fn get_struct(&self, field_id: FieldID) -> Option<&Vec<u8>> {
		self.structures.get(&field_id).map(|f| &f.data)
	}
	
	pub fn set_long_counter(&mut self, field_id: FieldID, value: i64) {
		self.long_counters.insert(field_id, LongCounter { counter: value });
	}
	
	pub fn get_long_counter(&self, field_id: FieldID) -> i64 {
		self.long_counters.get(&field_id).map(|f| f.counter).unwrap_or(0)
	}
	
	pub fn increment_long_counter(&mut self, field_id: FieldID, value: i64) -> i64 {
		let new_value = self.get_long_counter(field_id) + value;
		self.set_long_counter(field_id, new_value);
		return new_value;
	}
	
	pub fn set_float_counter(&mut self, field_id: FieldID, value: f64) {
		self.float_counters.insert(field_id, FloatCounter { counter: value });
	}
	
	pub fn get_float_counter(&self, field_id: FieldID) -> f64 {
		self.float_counters.get(&field_id).map(|f| f.counter).unwrap_or(0.0)
	}
	
	pub fn increment_float_counter(&mut self, field_id: FieldID, value: f64) -> f64 {
		let new_value = self.get_float_counter(field_id) + value;
		self.set_float_counter(field_id, new_value);
		return new_value;
	}
	
	
	pub fn send_event(&self, field_id: FieldID, event: Vec<u8>) {}
	
	pub fn to_global_object_id(client: &Client, local_object_id: u32) -> u64 {
		(client.configuration.id as u64).shl(32) + local_object_id as u64
	}
}


impl Room {
	pub fn object_increment_long_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: i64) -> i64 {
		let result = object.increment_long_counter(field_id, value);
		self.listener.on_object_long_counter_change(field_id, object);
		return result;
	}
	
	pub fn object_increment_float_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: f64) -> f64 {
		let result = object.increment_float_counter(field_id, value);
		self.listener.on_object_float_counter_change(field_id, object);
		return result;
	}
	
	pub fn object_update_struct(&mut self, object: &mut GameObject, field_id: FieldID, value: &Vec<u8>) {
		object.update_struct(field_id, value.clone());
		self.listener.on_object_struct_changed(field_id, object);
	}
}