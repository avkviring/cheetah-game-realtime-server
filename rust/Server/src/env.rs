#![cfg_attr(docsrs, feature(doc_cfg))]

use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use std::{panic, process};

pub use tonic;
use tonic::transport::Uri;
use tracing_log::LogTracer;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};

pub type StringId = heapless::String<20>;

#[must_use]
pub fn get_env(name: &str) -> String {
	std::env::var(name).unwrap_or_else(|_| panic!("Env {name} is not set"))
}

#[must_use]
pub fn get_env_or_default(name: &str, default: &str) -> String {
	std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

pub fn setup_tracer(trace_level: tracing::Level) {
	LogTracer::builder().init().unwrap();

	let fmt_layer = fmt::layer().with_target(false).with_ansi(false);
	let env_filter = EnvFilter::from_default_env().add_directive(Directive::from(trace_level));
	let subscriber = Registry::default().with(env_filter).with(fmt_layer);
	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[allow(clippy::print_stdout)]
#[allow(clippy::exit)]
pub fn setup_panic_hook() {
	panic::set_hook(Box::new(move |panic_info| {
		// ставим задачу на выход
		std::thread::spawn(|| {
			// ждем для сброса логов
			sleep(Duration::from_secs(2));
			// выходим
			process::exit(1)
		});
		println!("panic {panic_info}");
		// сообщаем об ошибке
		tracing::error!("{}", panic_info);
	}));
}

#[must_use]
pub fn get_internal_grpc_service_default_address() -> SocketAddr {
	format!("0.0.0.0:{}", get_internal_grpc_service_default_port()).parse().unwrap()
}

#[must_use]
pub fn get_internal_webgrpc_service_default_address() -> SocketAddr {
	format!("0.0.0.0:{}", get_internal_webgrpc_service_default_port()).parse().unwrap()
}

#[must_use]
pub fn get_public_htt11_service_binding_addr() -> SocketAddr {
	format!("0.0.0.0:{}", get_public_grpc_service_default_port()).parse().unwrap()
}

#[must_use]
pub fn get_admin_webgrpc_service_default_address() -> SocketAddr {
	format!("0.0.0.0:{}", get_admin_webgrpc_service_default_port()).parse().unwrap()
}

#[must_use]
pub fn get_public_grpc_service_default_port() -> u16 {
	5000
}
#[must_use]
pub fn get_internal_grpc_service_default_port() -> u16 {
	5001
}
#[must_use]
pub fn get_internal_webgrpc_service_default_port() -> u16 {
	6001
}

#[must_use]
pub fn get_admin_webgrpc_service_default_port() -> u16 {
	5002
}

#[must_use]
pub fn make_internal_srv_uri(host: &str, port: u16) -> Uri {
	format!("http://{host}:{port}").parse().unwrap()
}

#[must_use]
pub fn get_internal_srv_uri_from_env(service: &str) -> Uri {
	let host = get_env(format!("{service}_INTERNAL_SERVICE_HOST").as_str());
	let port_string = get_env(format!("{service}_INTERNAL_SERVICE_PORT").as_str());
	let port = match port_string.parse() {
		Ok(value) => value,
		Err(e) => {
			panic!("{service}_INTERNAL_SERVICE_PORT is not int {port_string} err={e:?}");
		}
	};
	make_internal_srv_uri(&host, port)
}
