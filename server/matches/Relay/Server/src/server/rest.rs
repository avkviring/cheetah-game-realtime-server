use std::future::Future;
use std::sync::{Arc, Mutex};
use warp::Filter;

use crate::server::RelayServer;

pub fn run_rest_server(server: Arc<Mutex<RelayServer>>) -> impl Future<Output = ()> {
	let dump = warp::path("dump").map(move || {
		let server = server.lock().unwrap();
		server.dump().unwrap().to_json()
	});
	warp::serve(dump).run(([0, 0, 0, 0], 8080))
}
