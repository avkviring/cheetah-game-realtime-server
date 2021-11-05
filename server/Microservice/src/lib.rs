#![cfg_attr(docsrs, feature(doc_cfg))]

use std::net::SocketAddr;

use log::LevelFilter;
pub use tonic;
use tonic::transport::Uri;

pub mod jwt;

pub fn get_env(name: &str) -> String {
	let value = std::env::var(name).unwrap_or_else(|_| panic!("Env {} dont set", name));
	if value.trim().is_empty() {
		panic!("Env {} is empty", name);
	}
	value
}

pub fn init(name: &str) {
	pretty_env_logger::formatted_timed_builder()
		.filter_level(LevelFilter::Info)
		.init();

	println!("start service {} ", name);
}

pub fn get_internal_service_binding_addr() -> SocketAddr {
	format!("0.0.0.0:{}", get_internal_service_port()).parse().unwrap()
}

pub fn get_external_service_binding_addr() -> SocketAddr {
	format!("0.0.0.0:{}", get_external_service_port()).parse().unwrap()
}
pub fn get_admin_service_binding_addr() -> SocketAddr {
	format!("0.0.0.0:{}", get_admin_service_port()).parse().unwrap()
}

pub fn get_external_service_port() -> u16 {
	5000
}

pub fn get_internal_service_port() -> u16 {
	5001
}

pub fn get_admin_service_port() -> u16 {
	5002
}

pub fn make_internal_srv_uri(host: &str, port: u16) -> Uri {
	format!("http://{}:{}", host, port).parse().unwrap()
}

pub fn get_internal_srv_uri_from_env(service: &str) -> Uri {
	let host = get_env(format!("{}_INTERNAL_SERVICE_HOST", service).as_str());
	let port_string = get_env(format!("{}_INTERNAL_SERVICE_PORT", service).as_str());
	let port = match port_string.parse() {
		Ok(value) => value,
		Err(_) => {
			panic!("{}_INTERNAL_SERVICE_PORT is not int {}", service, port_string);
		}
	};
	make_internal_srv_uri(&host, port)
}
