///
/// Сетевой клиент запускается в отдельном потоке, взаимодействие между потоками unity
/// (application) и сетевым клиентом осуществляется с помощью application_thread и network_thread
///
///
use std::time::Duration;

use crate::clients::network_thread::C2SCommandWithChannel;

pub mod application_thread;
pub mod network_thread;
pub mod registry;

#[derive(Debug)]
pub enum ClientRequest {
	SetProtocolTimeOffset(Duration),
	ConfigureRttEmulation(Duration, f64),
	ConfigureDropEmulation(f64, Duration),
	SendCommandToServer(C2SCommandWithChannel),
	ResetEmulation,
	Close,
}
