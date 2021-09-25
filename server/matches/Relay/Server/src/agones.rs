use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use cheetah_microservice::tonic::codegen::Arc;

use crate::server::RelayServer;

///
/// Взаимодействие с AGONES SDK
/// Если Agones  не запущен - то relay будет остановлен
///
pub async fn run_agones_cycle(halt_signal: Arc<AtomicBool>, relay_server: Arc<Mutex<RelayServer>>) {
	if !std::env::var("ENABLE_AGONES").is_ok() {
		return;
	}

	let (mut sdk, mut gameserver) = rymder::Sdk::connect(None, None, None)
		.await
		.expect("Agones: failed connect to SDK server");

	// сервер готов к работе
	sdk.mark_ready().await.unwrap();

	let mut health = sdk.health_check();

	let mut allocated = false;

	while !halt_signal.load(Ordering::Relaxed) {
		// при создании первой комнаты - вызываем allocate
		if !allocated && relay_server.lock().unwrap().created_room_counter > 0 {
			sdk.allocate().await.unwrap();
			allocated = true;
		}

		// подтверждаем что сервер жив
		if health.send(()).await.is_err() {
			log::error!("Agones: health receiver was closed");
			health = sdk.health_check();
		}
		tokio::time::sleep(Duration::from_secs(2)).await;
	}
	sdk.shutdown().await.unwrap();
}
