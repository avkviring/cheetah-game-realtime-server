use std::cell::RefCell;
use std::rc::Rc;

use indexmap::map::IndexMap;

use cheetah_relay_common::constants::{GlobalObjectId, LocalObjectId};
use cheetah_relay_common::room::access::{Access, AccessGroups};
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::room::clients::Client;
use crate::room::listener::RoomListener;
use crate::room::objects::object::{GameObject, ObjectFieldType};
use crate::room::objects::owner::Owner;
use crate::room::room::Room;

pub mod object;
pub mod owner;

/// Хранение и управление списком игровых объектов
pub struct Objects {
    objects: IndexMap<GlobalObjectId, Rc<RefCell<GameObject>>>,
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
        Objects {
            objects: Default::default(),
        }
    }
}

impl Objects {
    pub fn get(&self, id: GlobalObjectId) -> Option<Rc<RefCell<GameObject>>> {
        self.objects.get(&id).and_then(|f| Option::Some(f.clone()))
    }

    pub fn get_by_owner(&self, client: &Client, local_object_id: LocalObjectId) -> Option<Rc<RefCell<GameObject>>> {
        let id = GameObject::get_global_object_id_by_client(client, local_object_id);
        self.get(id)
    }

    pub fn len(&mut self) -> usize {
        return self.objects.len();
    }

    pub fn get_objects_by_owner(&mut self, owner: Owner) -> Vec<Rc<RefCell<GameObject>>> {
        self.objects
            .values()
            .filter(|o| {
                (*((*o).clone())).borrow().owner == owner
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
            .map(|o| {
                let o = o.clone();
                o
            })
            .collect::<Vec<_>>()
    }

    pub fn get_object_ids(&self) -> Vec<GlobalObjectId> {
        self
            .objects
            .keys()
            .map(|k| *k)
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
    pub fn create_client_game_object(&mut self,
                                     owner: &Client,
                                     local_object_id: LocalObjectId,
                                     access_group: AccessGroups,
                                     fields: GameObjectFields) -> Result<GlobalObjectId, CreateObjectError> {
        let client_groups = &owner.configuration.groups;
        if !client_groups.contains_any(&access_group) {
            return Result::Err(CreateObjectError::IncorrectGroups);
        }

        let object = GameObject::new(
            GameObject::get_global_object_id_by_client(owner, local_object_id),
            Owner::new_owner(owner),
            client_groups.clone(),
            fields,
        );
        let id = object.id;
        self.insert_game_object(object);
        Result::Ok(id)
    }

    ///
    /// Создание игрового объекта от root-а
    /// object_id - идентификатор объекта
    ///
    pub fn create_root_game_object(&mut self,
                                   object_id: u32,
                                   access_group: AccessGroups,
                                   fields: GameObjectFields,
    ) -> Result<u64, CreateObjectError> {
        let object = GameObject::new(
            object_id as u64,
            Owner::new_root_owner(),
            access_group,
            fields,
        );
        let id = object.id;
        self.insert_game_object(object);
        Result::Ok(id)
    }

    pub fn insert_game_object(&mut self, object: GameObject) -> GlobalObjectId {
        let id = object.id;
        self.listener.on_object_created(&object, &self.clients);
        self.objects.objects.insert(id, Rc::new(RefCell::new(object)));
        id
    }

    /// проверка прав доступа к полю объекта
    pub fn get_object_with_check_field_access(&mut self,
                                              _access: Access,
                                              _client: &Client,
                                              global_object_id: u64,
                                              _object_field_type: ObjectFieldType,
                                              _field_id: u16) ->
                                              Result<Rc<RefCell<GameObject>>, ErrorGetObjectWithCheckAccess> {
        let object = self.objects.get(global_object_id);
        match object {
            Some(object) => { Result::Ok(object) }
            None => { Result::Err(ErrorGetObjectWithCheckAccess::ObjectNotFound) }
        }
    }

    /// проверка прав доступа к полю объекта
    pub fn get_object_with_check_access(&self,
                                        _access: Access,
                                        _client: &Client,
                                        global_object_id: u64) ->
                                        Result<Rc<RefCell<GameObject>>, ErrorGetObjectWithCheckAccess> {
        let object = self.objects.get(global_object_id);
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



