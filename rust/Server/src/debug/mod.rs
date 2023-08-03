use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

use cheetah_protocol::others::member_id::MemberAndRoomId;

use crate::server::manager::ServerManager;

pub async fn run_debug_server(server_manager: Arc<Mutex<ServerManager>>, listener: TcpListener) {
	let cloned_server_manager = server_manager.clone();
	let dump = warp::path("get-rooms").map(move || get_rooms(cloned_server_manager.clone()));
	let cloned_server_manager = server_manager.clone();
	let delete_super_users = warp::path("delete-super-users").map(move || delete_super_users(cloned_server_manager.clone()));
	let cloned_server_manager = server_manager.clone();
	let disallow_delete_rooms = warp::path("disallow-delete-rooms").map(move || disallow_delete_rooms(cloned_server_manager.clone()));
	let cloned_server_manager = server_manager.clone();
	let allow_delete_rooms = warp::path("allow-delete-rooms").map(move || allow_delete_rooms(cloned_server_manager.clone()));
	let index = warp::any().map(get_help);

	let routes = dump.or(delete_super_users).or(disallow_delete_rooms).or(allow_delete_rooms).or(index);
	let stream = TcpListenerStream::new(listener);
	warp::serve(routes).run_incoming(stream).await
}

fn get_help() -> String {
	let mut result = String::new();
	result.push_str("/get-rooms - dump all rooms\n");
	result.push_str("/delete-super-users - delete all super users\n");
	result.push_str("/allow-delete-rooms - Allow delete rooms from grpc (default - allow)\n");
	result.push_str("/disallow-delete-rooms - Disallow delete rooms from grpc\n");
	result
}

fn disallow_delete_rooms(server_manager: Arc<Mutex<ServerManager>>) -> String {
	match server_manager.try_lock().unwrap().set_allow_delete_rooms(false) {
		Ok(_) => "Room deleted disallowed".to_string(),
		Err(e) => {
			format!("error {:?}", e)
		}
	}
}

fn allow_delete_rooms(server_manager: Arc<Mutex<ServerManager>>) -> String {
	match server_manager.try_lock().unwrap().set_allow_delete_rooms(true) {
		Ok(_) => "Room deleted allowed".to_string(),
		Err(e) => {
			format!("error {:?}", e)
		}
	}
}

fn delete_super_users(server_manager: Arc<Mutex<ServerManager>>) -> String {
	let mut result = String::new();
	let server_manager = server_manager.try_lock().unwrap();
	match server_manager.get_rooms() {
		Ok(rooms) => {
			for room_id in rooms {
				match server_manager.dump(room_id) {
					Ok(room) => {
						for (member_id, member) in room.unwrap().members {
							if member.template.super_member {
								let member_and_room_id = MemberAndRoomId { member_id, room_id };
								match server_manager.delete_member(member_and_room_id.clone()) {
									Ok(_) => result.push_str(format!("delete member {:?}\n", member_and_room_id).as_str()),
									Err(e) => {
										result.push_str(format!("delete member {:?} error {:?}\n", member_and_room_id, e).as_str());
									}
								}
							}
						}
					}
					Err(e) => {
						result.push_str(format!("delete member - dump room error {:?}\n", e).as_str());
					}
				}
			}
		}
		Err(e) => result.push_str(format!("delete member - get_rooms error {:?}\n", e).as_str()),
	}
	result
}

fn get_rooms(server_manager: Arc<Mutex<ServerManager>>) -> String {
	let server_manager = server_manager.try_lock().unwrap();
	match server_manager.get_rooms() {
		Ok(rooms) => {
			let rooms: Vec<_> = rooms.into_iter().map(|id| server_manager.dump(id).unwrap()).collect();
			match serde_json::to_string(&rooms) {
				Ok(s) => s,
				Err(e) => {
					format!("Error {:?}", e)
				}
			}
		}
		Err(e) => format!("{:?}", e),
	}
}
