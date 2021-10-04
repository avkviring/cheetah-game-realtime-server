use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use cheetah_microservice::tonic::codegen::Arc;

use crate::server::manager::RelayManager;

///
/// Взаимодействие с AGONES SDK
/// Если Agones  не запущен - то relay будет остановлен
///
pub async fn run_agones_cycle(halt_signal: Arc<AtomicBool>, relay_server: Arc<Mutex<RelayManager>>) {
	if std::env::var("ENABLE_AGONES").is_err() {
		return;
	}
	log::info!("Agones: Starting");
	match rymder::Sdk::connect(None, Option::Some(Duration::from_secs(2)), Option::Some(Duration::from_secs(2))).await {
		Ok((mut sdk, mut gameserver)) => {
			log::info!("Agones: Connected to SDK");
			// сервер готов к работе
			sdk.mark_ready().await.unwrap();
			log::info!("Agones: invoked sdk.mark_ready");

			let mut health = sdk.health_check();

			let mut allocated = false;

			while !halt_signal.load(Ordering::Relaxed) {
				// при создании первой комнаты - вызываем allocate
				if !allocated && relay_server.lock().unwrap().created_room_counter > 0 {
					sdk.allocate().await.unwrap();
					log::info!("Agones: invoked allocated");
					allocated = true;
				}

				// подтверждаем что сервер жив
				match health.send(()).await {
					Ok(_) => {
						log::info!("Agones: invoked health");
					}
					Err(e) => {
						log::error!("Agones: health receiver was closed {:?}", e);
						health = sdk.health_check();
					}
				}

				tokio::time::sleep(Duration::from_secs(2)).await;
			}
			sdk.shutdown().await.unwrap();
		}
		Err(e) => {
			log::error!("Agones: Fail connect {:?}", e);
			panic!("Agones: Fail connect {:?}", e);
		}
	}
}
