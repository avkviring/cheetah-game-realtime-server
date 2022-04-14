#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate core;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::Duration;
use std::{panic, process};

pub use tonic;
use tonic::transport::Uri;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
pub use tracing_subscriber::{fmt, EnvFilter, Layer, Registry};
use tracing_unwrap::ResultExt;

use crate::loki::LokiLayer;
use crate::prometheus::setup_prometheus;

pub mod jwt;
pub mod loki;
pub mod prometheus;

pub type StringId = heapless::String<20>;

pub fn init(name: &str) {
	setup_tracer(name);
	setup_panic_hook();
	setup_prometheus();
	tracing::info!("start service {} ", name);
}

pub fn get_env(name: &str) -> String {
	std::env::var(name).expect_or_log(format!("Env {} don't set", name).as_str())
}

fn setup_tracer(name: &str) {
	LogTracer::builder().with_max_level(log::LevelFilter::Info).init().unwrap();

	let fmt_layer = fmt::layer().with_target(false).with_ansi(false);

	let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
	let subscriber = Registry::default().with(env_filter).with(fmt_layer);

	if let Ok(loki_url) = std::env::var("LOKI_URL") {
		let mut default_values = HashMap::default();
		default_values.insert("service".to_owned(), name.to_owned());
		default_values.insert("namespace".to_owned(), get_env("NAMESPACE"));
		default_values.insert("hostname".to_owned(), get_env("HOSTNAME"));
		let loki_layer = LokiLayer::new(loki_url, default_values);
		let subscriber = subscriber.with(loki_layer);
		tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
	} else {
		tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
	}
}

fn setup_panic_hook() {
	panic::set_hook(Box::new(move |panic_info| {
		// ставим задачу на выход
		std::thread::spawn(|| {
			// ждем для сброса логов
			sleep(Duration::from_secs(2));
			// выходим
			process::exit(1)
		});
		// сообщаем об ошибке
		tracing::error!("{}", panic_info);
	}));
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
