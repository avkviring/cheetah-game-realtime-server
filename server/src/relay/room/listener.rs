use crate::relay::room::objects::object::{FieldID, GameObject};
use crate::relay::room::room::Room;

/// глобальный listener для обработки событий room
pub trait RoomListener {
	fn on_object_created(&mut self, game_object: &GameObject);
	fn on_object_delete(&mut self, game_object: &GameObject);
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject);
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject);
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &Vec<u8>, game_object: &GameObject);
	fn on_object_struct_changed(&mut self, field_id: FieldID, game_object: &GameObject);
}

pub struct CompositeRoomListener {
	listeners: Vec<Box<dyn RoomListener>>
}


impl RoomListener for CompositeRoomListener {
	fn on_object_created(&mut self, game_object: &GameObject) {
		for mut listener in self.listeners.iter_mut() {
			listener.on_object_created(game_object)
		}
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject) {
		for mut listener in self.listeners.iter_mut() {
			listener.on_object_delete(game_object)
		}
	}
	
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject) {
		for mut listener in self.listeners.iter_mut() {
			listener.on_object_long_counter_change(field_id, game_object)
		}
	}
	
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject) {
		for mut listener in self.listeners.iter_mut() {
			listener.on_object_float_counter_change(field_id, game_object)
		}
	}
	
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &Vec<u8>, game_object: &GameObject) {
		for mut listener in self.listeners.iter_mut() {
			listener.on_object_event_fired(field_id, event_data, game_object)
		}
	}
	
	fn on_object_struct_changed(&mut self, field_id: FieldID, game_object: &GameObject) {
		for mut listener in self.listeners.iter_mut() {
			listener.on_object_struct_changed(field_id, game_object)
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