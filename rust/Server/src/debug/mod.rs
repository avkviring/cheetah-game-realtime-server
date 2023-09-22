use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

use crate::room::Room;
use crate::server::manager::ServerManager;

pub async fn run_debug_server(server_manager: Arc<Mutex<ServerManager>>, listener: TcpListener) {
	let cloned_server_manager = server_manager.clone();

	let full_dump_rooms = warp::path("full_dump_rooms").map(move || full_dump_rooms(cloned_server_manager.clone()));
	let cloned_server_manager = server_manager.clone();
	let simple_dump_rooms = warp::path("simple_dump_rooms").map(move || simple_dump_rooms(cloned_server_manager.clone()));
	let index = warp::any().map(get_help);

	let routes = full_dump_rooms.or(simple_dump_rooms).or(index);
	let stream = TcpListenerStream::new(listener);
	warp::serve(routes).run_incoming(stream).await
}

fn get_help() -> String {
	let mut result = String::new();
	result.push_str("/full_dump_rooms - full dump rooms\n");
	result.push_str("/simple_dump_rooms - simple dump rooms\n");
	result
}

fn full_dump_rooms(server_manager: Arc<Mutex<ServerManager>>) -> String {
	let mut result = String::new();
	rooms_visitor(server_manager, |room| result.push_str(format!("{:?}", ron::to_string(&room)).as_str()));
	result
}

fn simple_dump_rooms(server_manager: Arc<Mutex<ServerManager>>) -> String {
	let mut result = String::new();
	rooms_visitor(server_manager, |room| result.push_str(format!("{:?}-{:?}/n", room.id, room.members.len()).as_str()));
	result
}

fn rooms_visitor<OnRoom>(server_manager: Arc<Mutex<ServerManager>>, mut on_room: OnRoom) -> String
where
	OnRoom: FnMut(Room),
{
	let mut result = String::new();
	let server_manager = server_manager.try_lock();
	if let Ok(server_manager) = server_manager {
		match server_manager.get_rooms() {
			Ok(rooms) => {
				let rooms: Vec<Room> = rooms.into_iter().map(|id| server_manager.dump(id)).filter_map(|item| item.ok()).filter_map(|item| item).collect();
				for room in rooms {
					on_room(room);
				}
			}
			Err(e) => result.push_str(format!("{:?}", e).as_str()),
		}
	} else {
		result.push_str("Cannot lock server_manager");
	}
	result
}

#[cfg(test)]
pub mod test {
	use std::sync::Arc;
	use cheetah_game_realtime_protocol::coniguration::ProtocolConfiguration;
	use tokio::sync::Mutex;
	use cheetah_common::network::bind_to_free_socket;
	use crate::debug::{full_dump_rooms, simple_dump_rooms};
	use crate::room::template::config::RoomTemplate;
	use crate::server::manager::ServerManager;

	#[test]
	pub fn test_simple_dump_rooms() {
		let mut server_manager = ServerManager::new(
			bind_to_free_socket().unwrap(),
			ProtocolConfiguration {
				disconnect_timeout: Default::default(),
			},
		)
		.unwrap();
		let room_id = server_manager
			.create_room(RoomTemplate {
				name: "".to_string(),
				objects: vec![],
			})
			.unwrap();

		let server_manager = Arc::new(Mutex::new(server_manager));
		let result = simple_dump_rooms(server_manager.clone());
		assert_eq!(result, format!("{:?}-0/n", room_id))
	}

	#[test]
	pub fn test_full_dump_rooms() {
		let mut server_manager = ServerManager::new(
			bind_to_free_socket().unwrap(),
			ProtocolConfiguration {
				disconnect_timeout: Default::default(),
			},
		)
		.unwrap();
		let room_id = server_manager
			.create_room(RoomTemplate {
				name: "perm_room".to_string(),
				objects: vec![],
			})
			.unwrap();

		let server_manager = Arc::new(Mutex::new(server_manager));
		let result = full_dump_rooms(server_manager);
		assert!(result.contains("perm_room"));
		assert!(result.contains(room_id.to_string().as_str()))
	}
}
