use std::cell::RefCell;
use std::rc::Rc;

use cheetah_relay_common::constants::FieldID;

use crate::room::clients::{Client, Clients};
use crate::room::objects::object::GameObject;
use crate::room::objects::Objects;
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;

/// глобальный listener для обработки событий room
pub trait RoomListener {
	#[allow(unused)]
	fn set_current_client(&mut self, client: Rc<Client>);
	#[allow(unused)]
	fn unset_current_client(&mut self);
	#[allow(unused)]
	fn set_current_meta_info(&mut self, meta: Rc<C2SMetaCommandInformation>);
	#[allow(unused)]
	fn unset_current_meta_info(&mut self);
	#[allow(unused)]
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {}
	#[allow(unused)]
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {}
	#[allow(unused)]
	fn on_client_connect(&mut self, client: &Client, objects: &Objects) {}
	#[allow(unused)]
	fn on_client_disconnect(&mut self, client: &Client) {}
	#[allow(unused)]
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {}
	#[allow(unused)]
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {}
	#[allow(unused)]
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &[u8], game_object: &GameObject, clients: &Clients) {}
	#[allow(unused)]
	fn on_object_struct_updated(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {}
	#[allow(unused)]
	fn on_object_long_counter_set(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients);
	#[allow(unused)]
	fn on_object_float_counter_set(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients);
}

pub struct CompositeRoomListener {
	pub listeners: Vec<Rc<RefCell<dyn RoomListener>>>
}

impl CompositeRoomListener {
	pub fn add_listener(&mut self, listener: Rc<RefCell<dyn RoomListener>>) {
		self.listeners.push(listener);
	}
}


impl RoomListener for CompositeRoomListener {
	fn set_current_client(&mut self, client: Rc<Client>) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().set_current_client(client.clone())
		}
	}
	
	fn unset_current_client(&mut self) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().unset_current_client()
		}
	}
	
	fn set_current_meta_info(&mut self, meta: Rc<C2SMetaCommandInformation>) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().set_current_meta_info(meta.clone());
		}
	}
	
	fn unset_current_meta_info(&mut self) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().unset_current_meta_info();
		}
	}
	
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_created(game_object, clients)
		}
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_delete(game_object, clients)
		}
	}
	
	fn on_client_connect(&mut self, client: &Client, objects: &Objects) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_client_connect(client, objects)
		}
	}
	
	fn on_client_disconnect(&mut self, client: &Client) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_client_disconnect(client)
		}
	}
	
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_long_counter_change(field_id, game_object, clients)
		}
	}
	
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_float_counter_change(field_id, game_object, clients)
		}
	}
	
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &[u8], game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_event_fired(field_id, event_data, game_object, clients)
		}
	}
	
	fn on_object_struct_updated(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_struct_updated(field_id, game_object, clients)
		}
	}
	
	fn on_object_long_counter_set(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_long_counter_set(field_id, game_object, clients)
		}
	}
	
	fn on_object_float_counter_set(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.borrow_mut().on_object_float_counter_set(field_id, game_object, clients)
		}
	}
}

impl Default for CompositeRoomListener {
	fn default() -> Self {
		CompositeRoomListener {
			listeners: Default::default()
		}
	}
}