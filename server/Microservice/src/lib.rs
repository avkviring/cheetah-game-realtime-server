use std::net::SocketAddr;

pub mod jwt;

pub fn get_env(name: &str) -> String {
    let value = std::env::var(name).unwrap_or_else(|_| panic!("Env {} dont set", name));
    if value.trim().is_empty() {
        panic!("Env {} is empty", name);
    }
    value
}

pub fn init(name: &str) {
    pretty_env_logger::init();
    println!("start service {} ", name);
}

pub fn get_internal_grpc_address() -> SocketAddr {
    "0.0.0.0:5001".parse().unwrap()
}
