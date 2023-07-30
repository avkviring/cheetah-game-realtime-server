use std::str::FromStr;
use std::time::Duration;

use fnv::FnvHashSet;

use cheetah_server::builder::ServerBuilder;
use cheetah_server::env::{
	get_debug_rest_service_default_address, get_env_or_default, get_internal_grpc_service_default_address, get_internal_webgrpc_service_default_address, setup_panic_hook, setup_tracer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	setup_tracer(tracing::Level::from_str(get_env_or_default("LOG_LEVEL", "info").as_str()).unwrap());
	setup_panic_hook();
	prometheus_measures_exporter::start_prometheus_exporter();
	tracing::info!("start server");

	let mut builder = ServerBuilder::default()
		.set_internal_grpc_service_bind_address(get_internal_grpc_service_default_address())
		.set_internal_webgrpc_service_bind_address(get_internal_webgrpc_service_default_address())
		.set_debug_rest_service_bind_address(get_debug_rest_service_default_address())
		.set_games_service_bind_address("0.0.0.0:5555".parse().unwrap())
		.set_plugin_names(get_plugin_names("PLUGIN_NAMES"))
		.set_disconnect_duration(Duration::from_secs(get_env_or_default("DISCONNECT_TIMEOUT_IN_SEC", "180").parse().unwrap()));

	if std::env::var("ENABLE_AGONES").is_ok() {
		builder = builder.enable_agones();
	}

	let server = builder.build().await.unwrap();
	server.run().await;

	Ok(())
}

fn get_plugin_names(env_var: &str) -> FnvHashSet<String> {
	// плагины должны быть в формате PLUGIN_NAMES=plugin_1;plugin_2
	get_env_or_default(env_var, "").split_terminator(';').map(ToString::to_string).collect()
}

#[cfg(test)]
mod tests {
	use std::env;

	use fnv::FnvHashSet;
	use rand::distributions::{Alphanumeric, DistString};

	use crate::get_plugin_names;

	#[test]
	fn test_get_plugin_names() {
		let env_var = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
		env::set_var(&env_var, "plugin_1;plugin_2");
		assert_eq!(FnvHashSet::<String>::from_iter(["plugin_1".to_owned(), "plugin_2".to_owned()]), get_plugin_names(&env_var));
		env::remove_var(&env_var);
	}

	#[test]
	fn test_get_plugin_names_empty() {
		let env_var = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
		let plugin_names = get_plugin_names(&env_var);
		assert!(plugin_names.is_empty(), "plugin_names are not empty: {plugin_names:?}");
	}
}
