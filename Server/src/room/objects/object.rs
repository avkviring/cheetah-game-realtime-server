use std::collections::HashMap;
use std::ops::Shl;

use cheetah_relay_common::constants::{ClientId, FieldID};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::room::clients::Client;
use crate::room::listener::RoomListener;
use crate::room::objects::owner::Owner;
use crate::room::room::Room;

/// Игровой объект
/// содержит данные от пользователей
#[derive(Debug, Clone, PartialEq)]
pub struct GameObject {
    pub id: u64,
    pub owner: Owner,
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
    pub fn new(id: u64, owner: Owner, access_groups: AccessGroups, fields: GameObjectFields) -> GameObject {
        GameObject {
            id,
            owner,
            access_groups,
            fields,
        }
    }

    pub fn update_struct(&mut self, field_id: FieldID, data: Vec<u8>) {
        self.fields.structures.insert(field_id, data);
    }

    pub fn get_struct(&self, field_id: FieldID) -> Option<&Vec<u8>> {
        self.fields.structures.get(&field_id)
    }

    pub fn set_long_counter(&mut self, field_id: FieldID, value: i64) {
        self.fields.long_counters.insert(field_id, value);
    }

    pub fn get_long_counter(&self, field_id: FieldID) -> i64 {
        *self.fields.long_counters.get(&field_id).unwrap_or(&Default::default())
    }

    pub fn increment_long_counter(&mut self, field_id: FieldID, value: i64) -> i64 {
        let new_value = self.get_long_counter(field_id) + value;
        self.set_long_counter(field_id, new_value);
        new_value
    }

    pub fn set_float_counter(&mut self, field_id: FieldID, value: f64) {
        self.fields.float_counters.insert(field_id, value);
    }

    pub fn get_float_counter(&self, field_id: FieldID) -> f64 {
        *self.fields.float_counters.get(&field_id).unwrap_or(&Default::default())
    }

    pub fn increment_float_counter(&mut self, field_id: FieldID, value: f64) -> f64 {
        let new_value = self.get_float_counter(field_id) + value;
        self.set_float_counter(field_id, new_value);
        new_value
    }


    pub fn send_event(&self, _field_id: FieldID, _event: &Vec<u8>) {}

    pub fn get_global_object_id_by_client(client: &Client, local_object_id: u32) -> u64 {
        GameObject::get_global_object_id_by_client_id(client.configuration.id, local_object_id)
    }
    pub fn get_global_object_id_by_client_id(client_id: ClientId, local_object_id: u32) -> u64 {
        (client_id as u64).shl(32) + local_object_id as u64
    }
}


impl Room {
    pub fn object_increment_long_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: i64) -> i64 {
        let result = object.increment_long_counter(field_id, value);
        self.listener.on_object_long_counter_change(field_id, object, &self.clients);
        result
    }

    pub fn object_set_long_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: i64) -> i64 {
        unimplemented!();
    }

    pub fn object_increment_float_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: f64) -> f64 {
        let result = object.increment_float_counter(field_id, value);
        self.listener.on_object_float_counter_change(field_id, object, &self.clients);
        result
    }

    pub fn object_set_float_counter(&mut self, object: &mut GameObject, field_id: FieldID, value: f64) -> f64 {
        unimplemented!()
    }

    pub fn object_update_struct(&mut self, object: &mut GameObject, field_id: FieldID, value: Vec<u8>) {
        object.update_struct(field_id, value);
        self.listener.on_object_struct_updated(field_id, object, &self.clients);
    }

    pub fn object_send_event(&mut self, object: &mut GameObject, field_id: FieldID, event_data: &Vec<u8>) {
        object.send_event(field_id, event_data);
        self.listener.on_object_event_fired(field_id, event_data, object, &self.clients);
    }
}
