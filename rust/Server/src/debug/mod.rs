use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

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
	let server_manager = server_manager.try_lock().unwrap();
	match server_manager.get_rooms() {
		Ok(rooms) => {
			let rooms: Vec<_> = rooms.into_iter().map(|id| server_manager.dump(id).unwrap()).collect();
			match ron::to_string(&rooms) {
				Ok(s) => s,
				Err(e) => {
					format!("Error {:?}", e)
				}
			}
		}
		Err(e) => format!("{:?}", e),
	}
}

fn simple_dump_rooms(server_manager: Arc<Mutex<ServerManager>>) -> String {
	let mut result = String::new();
	let server_manager = server_manager.try_lock().unwrap();
	match server_manager.get_rooms() {
		Ok(rooms) => {
			let rooms: Vec<_> = rooms.into_iter().map(|id| server_manager.dump(id).unwrap()).collect();
			for room in rooms {
				if let Some(room) = room {
					result.push_str(format!("{:?}-{:?}/n", room.id, room.members.len()).as_str())
				}
			}
		}
		Err(e) => result.push_str(format!("{:?}", e).as_str()),
	}
	result
}
