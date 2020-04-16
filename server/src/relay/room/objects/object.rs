use std::collections::HashMap;
use std::ops::Shl;

use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::owner::Owner;

/// Игровой объект
/// содержит данные от пользователей
pub struct GameObject {
	pub id: u64,
	pub owner: Owner,
	/// счетчики
	counters: HashMap<u16, DataCounter>,
	/// структуры (для сервера это массивы данных)
	structures: HashMap<u16, DataStruct>,
	/// группы доступа
	pub groups: AccessGroups,
}


/// счетчик
pub struct DataCounter {
	counter: i64
}

/// данные
pub struct DataStruct {
	data: Vec<u8>
}

#[derive(Debug)]
pub enum ObjectFieldType {
	LongCounter,
	FloatCounter,
	Struct,
	StringToIdMap,
	IdSet,
}


impl GameObject {
	pub fn new_client_object(client: &Client, local_object_id: u32, groups: AccessGroups) -> GameObject {
		GameObject {
			id: GameObject::to_global_object_id(client, local_object_id),
			owner: Owner::new_owner(client),
			counters: Default::default(),
			structures: Default::default(),
			groups,
		}
	}
	
	pub fn new_root_object(id: u64, groups: AccessGroups) -> GameObject {
		GameObject {
			id,
			owner: Owner::new_root_owner(),
			counters: Default::default(),
			structures: Default::default(),
			groups,
		}
	}
	
	pub fn update_struct(&mut self, struct_id: u16, data: Vec<u8>) {
		self.structures.insert(struct_id, DataStruct { data });
	}
	
	pub fn get_struct(&self, struct_id: u16) -> Option<&Vec<u8>> {
		self.structures.get(&struct_id).map(|f| &f.data)
	}
	
	pub fn set_counter(&mut self, counter_id: u16, value: i64) {
		self.counters.insert(counter_id, DataCounter { counter: value });
	}
	
	pub fn get_counter(&mut self, counter_id: u16) -> i64 {
		self.counters.get(&counter_id).map(|f| f.counter).unwrap_or(0)
	}
	
	pub fn increment_counter(&mut self, counter_id: u16, value: i64) {
		let current = self.get_counter(counter_id);
		self.set_counter(counter_id, current + value)
	}
	
	pub fn to_global_object_id(client: &Client, local_object_id: u32) -> u64 {
		(client.configuration.id as u64).shl(32) + local_object_id as u64
	}
}

