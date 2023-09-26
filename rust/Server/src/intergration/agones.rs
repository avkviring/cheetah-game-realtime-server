use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use rymder::Sdk;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use crate::intergration::registry::notify_registry_with_tracing_error;
use crate::intergration::registry::proto::status::State;
use crate::server::manager::ServerManager;


///
/// Цикл оповещения agones и NotifyService
///
pub async fn agones_und_notifyservice_cycle(server_manager: Arc<Mutex<ServerManager>>, max_alive_rooms: usize, max_created_rooms: usize) {
    match Sdk::connect(None, Some(Duration::from_secs(2)), Some(Duration::from_secs(2))).await {
        Ok((mut sdk, gameserver)) => {
            sdk.mark_ready().await.unwrap();

            let mut health = sdk.health_check();
            let mut allocated = false;

            while is_server_running(&server_manager).await {
                let server_manager = server_manager.lock().await;
                let current_count_rooms = server_manager.get_rooms().unwrap_or_default().len();
                let created_rooms_count = server_manager.get_created_rooms_count().unwrap();

                if !allocated && current_count_rooms > 0 {
                    sdk.allocate().await.unwrap();
                    allocated = true;
                }

                let need_to_restart = created_rooms_count >= max_created_rooms;
                let is_allow_to_restart = need_to_restart && current_count_rooms == 0;
                if is_allow_to_restart {
                    tracing::info!("Max created rooms limit reached - shutdown");
                    sdk.shutdown().await.unwrap();
                    notify_registry_with_tracing_error(&gameserver, State::NotReady).await;
                } else {
                    let server_is_full = current_count_rooms >= max_alive_rooms;
                    let state = if allocated {
                        if server_is_full || need_to_restart {
                            State::NotReady
                        } else {
                            State::Allocated
                        }
                    } else {
                        State::Ready
                    };
                    notify_registry_with_tracing_error(&gameserver, state).await;
                    health = send_agones_health(&mut sdk, health).await;
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            notify_registry_with_tracing_error(&gameserver, State::NotReady).await;
            sdk.shutdown().await.unwrap();
        }
        Err(e) => {
            panic!("Agones: Fail connect {e:?}");
        }
    }
}

async fn send_agones_health(sdk: &mut Sdk, health: Sender<()>) -> Sender<()> {
    let result = match health.send(()).await {
        Ok(_) => health,
        Err(e) => {
            tracing::error!("Agones: health receiver was closed {:?}", e);
            sdk.health_check()
        }
    };
    tracing::debug!("Agones: sended agones health signal");
    result
}

async fn is_server_running(server_manager: &Arc<Mutex<ServerManager>>) -> bool {
    !server_manager.lock().await.get_halt_signal().load(Ordering::Relaxed)
}
