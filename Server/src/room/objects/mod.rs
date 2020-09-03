use std::cell::RefCell;
use std::rc::Rc;

use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;
use indexmap::map::IndexMap;

use crate::room::clients::Client;
use crate::room::listener::RoomListener;
use crate::room::objects::id::{ServerGameObjectId, ServerOwner};
use crate::room::objects::object::GameObject;
use crate::room::Room;

pub mod object;
pub mod id;

/// Хранение и управление списком игровых объектов
pub struct Objects {
	objects: IndexMap<ServerGameObjectId, Rc<RefCell<GameObject>>>,
}

#[derive(Debug)]
pub enum ErrorGetObjectWithCheckAccess {
	ObjectNotFound
}


#[derive(Debug)]
pub enum GameObjectCreateErrors {
	AlreadyExists(ServerGameObjectId)
}

impl Default for Objects {
	fn default() -> Self {
		Objects {
			objects: Default::default(),
		}
	}
}

impl Objects {
	pub fn get(&self, id: &ServerGameObjectId) -> Option<Rc<RefCell<GameObject>>> {
		self.objects.get(id).and_then(|f| Option::Some(f.clone()))
	}
	
	pub fn len(&mut self) -> usize {
		self.objects.len()
	}
	
	pub fn get_objects_by_owner(&mut self, owner: ServerOwner) -> Vec<Rc<RefCell<GameObject>>> {
		self.objects
			.values()
			.filter(|o| {
				(*((*o).clone())).borrow().id.owner == owner
			})
			.map(|o| (*o).clone())
			.collect()
	}
	
	/// Получить объекты для группы в порядке их создания
	pub fn get_objects_by_group_in_create_order(&self, access_group: &AccessGroups) -> Vec<Rc<RefCell<GameObject>>> {
		// полный перебор объектов
		// но из-за сессионной природы битвы возможно этот вариант быстрее чем постоянно формировать
		// структуры для быстрого поиска объектов
		self
			.objects
			.values()
			.filter(|&o| {
				let o = o.clone();
				let o = &*o;
				let o = o.borrow();
				o.access_groups.contains_any(access_group)
			})
			.cloned()
			.collect::<Vec<_>>()
	}
	
	pub fn get_object_ids(&self) -> Vec<ServerGameObjectId> {
		self
			.objects
			.keys()
			.cloned()
			.collect()
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
	pub fn new_game_object(&mut self,
						   object_id: ServerGameObjectId,
						   template: u16,
						   access_group: AccessGroups,
						   fields: GameObjectFields) -> Result<(), GameObjectCreateErrors> {
		let object = GameObject::new(
			object_id,
			template,
			access_group,
			fields,
		);
		self.insert_game_object(object)
	}
	
	
	pub fn insert_game_object(&mut self, object: GameObject) -> Result<(), GameObjectCreateErrors> {
		match self.objects.objects.get(&object.id) {
			None => {
				let id = object.id.clone();
				self.listener.on_object_created(&object, &self.clients);
				self.objects.objects.insert(id, Rc::new(RefCell::new(object)));
				Result::Ok(())
			}
			Some(_) => {
				Result::Err(GameObjectCreateErrors::AlreadyExists(object.id.clone()))
			}
		}
	}
	
	/// проверка прав доступа к полю объекта
	pub fn get_object_with_check_field_access(&mut self,
											  client: &Client,
											  object_id: &ClientGameObjectId) ->
											  Result<Rc<RefCell<GameObject>>, ErrorGetObjectWithCheckAccess> {
		let object_id = ServerGameObjectId::from_client_object_id(Option::Some(client.configuration.id), object_id);
		let object = self.objects.get(&object_id);
		match object {
			Some(object) => { Result::Ok(object) }
			None => { Result::Err(ErrorGetObjectWithCheckAccess::ObjectNotFound) }
		}
	}
	
	/// проверка прав доступа к полю объекта
	pub fn get_object_with_check_access(&self,
										client: &Client,
										object_id: &ClientGameObjectId) ->
										Result<Rc<RefCell<GameObject>>, ErrorGetObjectWithCheckAccess> {
		let game_object_id = ServerGameObjectId::from_client_object_id(Option::Some(client.configuration.id), object_id);
		let object = self.objects.get(&game_object_id);
		match object {
			Some(object) => { Result::Ok(object) }
			None => { Result::Err(ErrorGetObjectWithCheckAccess::ObjectNotFound) }
		}
	}
	
	pub fn delete_game_object(&mut self, game_object: &GameObject) {
		self.listener.on_object_delete(game_object, &self.clients);
		self.objects.objects.remove(&game_object.id);
	}
}



