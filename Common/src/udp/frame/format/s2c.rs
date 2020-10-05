use serde::{Deserialize, Serialize};

///
/// Пакеты с сервера на клиент
///
#[derive(Debug, Serialize, Deserialize)]
pub enum S2CFrame {
	AckConnection(AckConnectionS2CPacket),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AckConnectionS2CPacket {}

