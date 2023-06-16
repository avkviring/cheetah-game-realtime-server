use std::sync::atomic::AtomicU64;
use std::sync::Arc;
///
/// Сетевой клиент запускается в отдельном потоке, взаимодействие между потоками unity
/// (application) и сетевым клиентом осуществляется с помощью `application_thread` и `network_thread`
///
///
use std::time::Duration;

use cheetah_protocol::disconnect::command::DisconnectByCommandReason;

use crate::clients::network_thread::C2SCommandWithChannel;

pub mod application_thread;
pub mod network_thread;
pub mod registry;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ClientRequest {
	SetProtocolTimeOffsetForTest(Duration),
	ConfigureRttEmulation(Duration, f64),
	ConfigureDropEmulation(f64, Duration),
	SendCommandToServer(C2SCommandWithChannel),
	ResetEmulation,
	Close(DisconnectByCommandReason),
}

///
/// Общая статистика между сетевым потоком и потоком приложения
///
#[derive(Debug, Clone, Default)]
pub struct SharedClientStatistics {
	pub current_frame_id: Arc<AtomicU64>,
	pub rtt_in_ms: Arc<AtomicU64>,
	pub recv_packet_count: Arc<AtomicU64>,
	pub send_packet_count: Arc<AtomicU64>,
	pub recv_size: Arc<AtomicU64>,
	pub send_size: Arc<AtomicU64>,
}
