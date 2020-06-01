use std::sync::mpsc::Sender;

use crate::client::command::{C2SCommandUnion, S2CCommandUnion};

pub enum ClientRequestType {
	GetS2CCommands(Sender<Vec<S2CCommandUnion>>),
	Close,
	SendCommandToServer(C2SCommandUnion),
}


///
/// Обработка внешних запросов
///
struct ExternalRequestProcessor {}
