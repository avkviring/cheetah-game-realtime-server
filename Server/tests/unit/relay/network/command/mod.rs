use std::rc::Rc;

use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::ClientGameObjectId;

use cheetah_relay::room::clients::Client;
use cheetah_relay::room::objects::id::{ServerGameObjectId, ServerOwner};
use cheetah_relay::room::Room;

pub mod c2s;
pub mod s2c;

pub mod unload;
pub mod structure;
pub mod long_counter;
pub mod float_counter;
pub mod upload;

fn create_game_object(room: &mut Room, client: &Rc<Client>) -> (ServerGameObjectId, ClientGameObjectId) {
	let server_object_id = ServerGameObjectId::new(0, ServerOwner::Client(client.configuration.id));
	let client_object_id = server_object_id.to_client_object_id(Option::Some(client.configuration.id));
	room.new_game_object(
		server_object_id.clone(),
		123,
		AccessGroups::from(0b10_0000),
		Default::default(),
	).unwrap();
	(server_object_id, client_object_id)
}