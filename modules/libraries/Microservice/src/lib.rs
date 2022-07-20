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
use tracing_loki_layer::LokiLayer;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::layer::SubscriberExt;
pub use tracing_subscriber::{fmt, EnvFilter, Layer, Registry};

pub mod trace;

pub type StringId = heapless::String<20>;

pub fn init(name: &str) {
	init_with_trace_level(name, tracing::Level::INFO)
}

pub fn init_with_trace_level(name: &str, trace_level: tracing::Level) {
	setup_tracer(name, trace_level);
	setup_panic_hook();
	prometheus_measures_exporter::start_prometheus_exporter();
	tracing::info!("start service {} ", name);
}

pub fn get_env(name: &str) -> String {
	std::env::var(name).unwrap_or_else(|_| panic!("Env {} is not set", name))
}

pub fn get_env_or_default(name: &str, default: &str) -> String {
	std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn setup_tracer(name: &str, trace_level: tracing::Level) {
	LogTracer::builder().init().unwrap();

	let fmt_layer = fmt::layer().with_target(false).with_ansi(false);

	let env_filter = EnvFilter::from_default_env().add_directive(Directive::from(trace_level));
	let subscriber = Registry::default().with(env_filter).with(fmt_layer);
	if let Ok(loki_url) = std::env::var("LOKI_URL") {
		let mut default_values = HashMap::default();
		default_values.insert("source".to_owned(), "server".to_owned());
		default_values.insert("type".to_owned(), "log".to_owned());
		default_values.insert("service".to_owned(), name.to_owned());
		default_values.insert("namespace".to_owned(), get_env("NAMESPACE"));
		default_values.insert("hostname".to_owned(), get_env("HOSTNAME"));
		let loki_layer = LokiLayer::new(loki_url, default_values);
		let subscriber = subscriber.with(loki_layer);
		tracing::subscriber::set_global_default(subscriber)
			.expect("setting default subscriber failed");
	} else {
		tracing::subscriber::set_global_default(subscriber)
			.expect("setting default subscriber failed");
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
	format!("0.0.0.0:{}", get_internal_service_port())
		.parse()
		.unwrap()
}

pub fn get_external_service_binding_addr() -> SocketAddr {
	format!("0.0.0.0:{}", get_external_service_port())
		.parse()
		.unwrap()
}
pub fn get_admin_service_binding_addr() -> SocketAddr {
	format!("0.0.0.0:{}", get_admin_service_port())
		.parse()
		.unwrap()
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
			panic!(
				"{}_INTERNAL_SERVICE_PORT is not int {}",
				service, port_string
			);
		}
	};
	make_internal_srv_uri(&host, port)
}
