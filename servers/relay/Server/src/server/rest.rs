use std::{io, thread};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use actix_rt::System;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use actix_web::body::Body;
use actix_web::http::header;

use crate::server::Server;

#[derive(Default)]
pub struct DumpRestServer {}

impl DumpRestServer {
	pub fn run(server: Arc<Mutex<Server>>) -> JoinHandle<io::Result<()>> {
		thread::spawn(move || {
			let sys = System::new("dump-rest");
			let server_data = web::Data::new(server);
			HttpServer::new(
				move || {
					App::new()
						.wrap(middleware::Compress::default())
						.app_data(server_data.clone())
						.route("/", web::get().to(DumpRestServer::index))
						.route("/dump", web::get().to(DumpRestServer::dump))
				})
				.workers(1)
				.bind("0.0.0.0:8080")?
				.shutdown_timeout(1)
				.run();
			
			sys.run()
		})
	}
	
	async fn dump(data: web::Data<Arc<Mutex<Server>>>) -> HttpResponse {
		let server = data.get_ref().lock().unwrap();
		HttpResponse::Ok()
			.header(header::CONTENT_TYPE, "application/json")
			.body(Body::from(server.dump().unwrap().to_json()))
	}
	
	async fn index() -> HttpResponse {
		let body = r#"
			<a href="/dump">Dump all server state</a><br/>
		"#;
		HttpResponse::Ok()
			.header(header::CONTENT_TYPE, "text/html")
			.body(Body::from(body))
	}
}

