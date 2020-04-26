use crate::relay::room::clients::{Client, Clients};
/// глобальный listener для обработки событий room
use crate::relay::room::objects::object::{FieldID, GameObject};
use crate::relay::room::objects::Objects;

pub trait RoomListener {
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients);
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients);
	fn on_client_connect(&mut self, client: &Client,  objects: &Objects);
	fn on_client_disconnect(&mut self, client: &Client);
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients);
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients);
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &Vec<u8>, game_object: &GameObject, clients: &Clients);
	fn on_object_struct_updated(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients);
}

pub struct CompositeRoomListener {
	listeners: Vec<Box<dyn RoomListener>>
}

impl CompositeRoomListener {
	pub fn add_listener(&mut self, listener: Box<dyn RoomListener>) {
		self.listeners.push(listener);
	}
}


impl RoomListener for CompositeRoomListener {
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.on_object_created(game_object, clients)
		}
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.on_object_delete(game_object, clients)
		}
	}
	
	fn on_client_connect(&mut self, client: &Client,  objects: &Objects) {
		for listener in self.listeners.iter_mut() {
			listener.on_client_connect(client, objects)
		}
	}
	
	fn on_client_disconnect(&mut self, client: &Client) {
		for listener in self.listeners.iter_mut() {
			listener.on_client_disconnect(client)
		}
	}
	
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.on_object_long_counter_change(field_id, game_object, clients)
		}
	}
	
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.on_object_float_counter_change(field_id, game_object, clients)
		}
	}
	
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &Vec<u8>, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.on_object_event_fired(field_id, event_data, game_object, clients)
		}
	}
	
	fn on_object_struct_updated(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		for listener in self.listeners.iter_mut() {
			listener.on_object_struct_updated(field_id, game_object, clients)
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