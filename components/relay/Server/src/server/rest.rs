use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, thread};

use actix_rt::System;
use actix_web::body::Body;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};

use crate::server::Server;
use cheetah_relay_common::room::RoomId;

#[derive(Default)]
pub struct RestServer {}

impl RestServer {
	pub fn run(server: Arc<Mutex<Server>>) -> JoinHandle<io::Result<()>> {
		thread::spawn(move || {
			let sys = System::new("rest");
			let server_data = web::Data::new(server);
			HttpServer::new(move || {
				App::new()
					.wrap(middleware::Compress::default())
					.app_data(server_data.clone())
					.route("/", web::get().to(RestServer::index))
					.route("/dump", web::get().to(RestServer::dump))
					.route("/select-user/{room}", web::get().to(RestServer::get_user_for_entrance))
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

	async fn get_user_for_entrance(room: web::Path<RoomId>, data: web::Data<Arc<Mutex<Server>>>) -> HttpResponse {
		let server = data.get_ref().lock().unwrap();

		let result = server.select_user_for_entrance(room.0).unwrap();
		HttpResponse::Ok()
			.header(header::CONTENT_TYPE, "application/json")
			.body(Body::from(serde_json::to_string(&result).unwrap()))
	}

	async fn index() -> HttpResponse {
		let body = r#"
			<a href="/dump">Dump all server state</a><br/>
			<a href="/select-user">Get user public/private key for enter.</a><br/>
		"#;
		HttpResponse::Ok().header(header::CONTENT_TYPE, "text/html").body(Body::from(body))
	}
}
