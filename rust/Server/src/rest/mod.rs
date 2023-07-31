use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Filter;

use crate::server::manager::ServerManager;

pub async fn run_rest_server(server_manager: Arc<Mutex<ServerManager>>, listener: TcpListener) {
	let routes = warp::any().map(move || {
		let server_manager = server_manager.try_lock().unwrap();
		match server_manager.get_rooms() {
			Ok(rooms) => {
				let rooms: Vec<_> = rooms.into_iter().map(|id| server_manager.dump(id).unwrap()).collect();
				format!("{:#?}", rooms)
			}
			Err(e) => format!("{:?}", e),
		}
	});

	let stream = TcpListenerStream::new(listener);
	warp::serve(routes).run_incoming(stream).await
}
