use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::relay::room::clients::Client;
use crate::relay::room::groups::{Access, AccessGroups};
use crate::relay::room::objects::object::{GameObject, ObjectFieldType};
use crate::relay::room::objects::owner::Owner;
use crate::relay::room::room::{GlobalObjectId, LocalObjectId, Room};

pub mod object;
pub mod owner;

/// Хранение и управление списком игровых объектов
pub struct Objects {
	objects: HashMap<GlobalObjectId, Rc<RefCell<GameObject>>>,
}

#[derive(Debug)]
pub enum CreateObjectError {
	IncorrectGroups
}

#[derive(Debug)]
pub enum ErrorGetObjectWithCheckAccess {
	ObjectNotFound,
	AccessNotAllowed,
}


impl Default for Objects {
	fn default() -> Self {
		return Objects {
			objects: Default::default(),
		};
	}
}

impl Objects {
	pub fn create_client_game_object(&mut self, owner: &Client, local_object_id: LocalObjectId, groups: AccessGroups) -> GlobalObjectId {
		let object = GameObject::new_client_object(owner, local_object_id, groups);
		return self.insert(object);
	}
	
	pub fn create_root_game_object(&mut self, id: LocalObjectId, groups: AccessGroups) -> GlobalObjectId {
		let object = GameObject::new_root_object(id as GlobalObjectId, groups);
		return self.insert(object);
	}
	
	pub fn insert(&mut self, object: GameObject) -> GlobalObjectId {
		let id = object.id;
		self.objects.insert(id, Rc::new(RefCell::new(object)));
		return id;
	}
	
	pub fn get(&self, id: GlobalObjectId) -> Option<Rc<RefCell<GameObject>>> {
		return self.objects.get(&id).and_then(|f| Option::Some(f.clone()));
	}
	
	pub fn get_by_owner(&self, client: &Client, local_object_id: LocalObjectId) -> Option<Rc<RefCell<GameObject>>> {
		let id = GameObject::to_global_object_id(client, local_object_id);
		return self.get(id);
	}
	
	pub fn len(&mut self) -> usize {
		return self.objects.len();
	}
	
	pub fn delete_objects_by_owner(&mut self, owner: Owner) {
		let object_for_remove: Vec<GlobalObjectId> = self.objects
			.values()
			.filter(|o| {
				(*((*o).clone())).borrow().owner == owner
			})
			.map(|o| (*(*o).clone()).borrow().id)
			.collect();
		
		for object_id in object_for_remove {
			self.objects.remove(&object_id);
		}
	}
	
	pub fn delete_object(&mut self, global_object_id: GlobalObjectId) {
		self.objects.remove(&global_object_id);
	}
}


impl Room {
	/// Создание клиентского игрового объекта
	///
	/// # Arguments
	/// * `owner` - владелец
	/// * `local_object_id` - идентификатор объекта в рамках клиента
	/// * `groups` - группы доступа
	///
	pub fn create_client_game_object(&mut self,
									 owner: &Client,
									 local_object_id: LocalObjectId,
									 groups: Option<AccessGroups>) -> Result<GlobalObjectId, CreateObjectError> {
		let client_groups = &owner.configuration.groups;
		let object_groups = if groups.is_none() {
			owner.configuration.groups.clone()
		} else {
			let groups = groups.unwrap();
			if !client_groups.contains_groups(&groups) {
				return Result::Err(CreateObjectError::IncorrectGroups);
			}
			groups
		};
		
		Result::Ok(self.objects.create_client_game_object(&owner, local_object_id, object_groups))
	}
	
	/// Создание игрового объекта от root-а
	/// object_id - идентификатор объекта
	pub fn create_root_game_object(&mut self, object_id: u32, groups: AccessGroups) -> Result<u64, CreateObjectError> {
		let id = self.objects.create_root_game_object(object_id, groups);
		Result::Ok(id)
	}
	
	/// проверка прав доступа к полю объекта
	/// сделано через room так как надо проверять права администратора
	pub fn get_object_with_check_field_access(&mut self,
											  access: Access,
											  client: &Client,
											  global_object_id: u64,
											  object_field_type: ObjectFieldType,
											  field_id: u16) ->
											  Result<&mut GameObject, ErrorGetObjectWithCheckAccess> {
		// let object = self.objects.get_mut(global_object_id);
		// return if object.is_some() {
		// 	Result::Ok(object.unwrap())
		// } else {
		// 	Result::Err(ErrorGetObjectWithCheckAccess::ObjectNotFound)
		// };
		unimplemented!();
	}
	
	/// проверка прав доступа к полю объекта
	/// сделано через room так как надо проверять права администратора
	pub fn get_object_with_check_access_mut(&mut self,
											access: Access,
											client: &Client,
											global_object_id: u64) ->
											Result<&mut GameObject, ErrorGetObjectWithCheckAccess> {
		// let object = self.objects.get_mut(global_object_id);
		// return if object.is_some() {
		// 	Result::Ok(object.unwrap())
		// } else {
		// 	Result::Err(ErrorGetObjectWithCheckAccess::ObjectNotFound)
		// };
		unimplemented!();
	}
	
	/// проверка прав доступа к полю объекта
	/// сделано через room так как надо проверять права администратора
	pub fn get_object_with_check_access(&self,
										access: Access,
										client: &Client,
										global_object_id: u64) ->
										Result<&GameObject, ErrorGetObjectWithCheckAccess> {
		// let object = self.objects.get(global_object_id);
		// return if object.is_some() {
		// 	Result::Ok(object.unwrap())
		// } else {
		// 	Result::Err(ErrorGetObjectWithCheckAccess::ObjectNotFound)
		// };
		unimplemented!()
	}
	
	pub fn delete_game_object(&mut self, game_object: &GameObject) {
		self.objects.delete_object(game_object.id)
	}
}



