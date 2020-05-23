// use actix_web::{App, get, HttpServer, Responder};
// use std::env;
//
//
// pub struct RestService {
//
// }
//
// #[get("/")]
// async fn index() -> impl Responder {
// 	format!("Hello!")
// }
//
// #[actix_rt::main]
// pub async fn init_rest() {
// 	env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
//
// 	let result = HttpServer::new(|| App::new().service(index))
// 		.bind("127.0.0.1:8080");
//
//
//
// 	result
// 		.unwrap()
// 		.run()
// 		.await;
// }
//
