#![cfg_attr(docsrs, feature(doc_cfg))]

use log::LevelFilter;
use std::net::SocketAddr;
use tonic::transport::Uri;

pub mod jwt;
pub mod proto;
pub use tonic;

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

pub fn get_self_service_internal_grpc_address() -> SocketAddr {
    "0.0.0.0:5001".parse().unwrap()
}

pub fn make_internal_grpc_uri(host: &str, port: u16) -> Uri {
    format!("http://{}:{}", host, port).parse().unwrap()
}
