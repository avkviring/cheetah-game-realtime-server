use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

use crate::server::manager::ServerManager;

pub async fn run_debug_server(server_manager: Arc<Mutex<ServerManager>>, listener: TcpListener) {
	let dump_method = warp::path("get-rooms").map(move || get_rooms(server_manager.clone()));
	let index = warp::any().map(get_help);

	let routes = dump_method.or(index);
	let stream = TcpListenerStream::new(listener);
	warp::serve(routes).run_incoming(stream).await
}

fn get_help() -> String {
	"/get-rooms/ - dump all rooms".to_string()
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
