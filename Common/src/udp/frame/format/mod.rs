use serde::{Deserialize, Serialize};

use crate::network::hash::UserPublicKey;

pub mod c2s;
pub mod s2c;


///
/// Структура для передачи через UDP
///
#[derive(Debug, PartialEq)]
pub struct UdpFrame {
	pub header: UdpFrameHeader,
	pub additional_headers: Vec<UdpAdditionalHeader>,
	pub commands: Vec<ApplicationCommand>,
}

impl UdpFrame {
	pub const VERSION: u8 = 0;
	pub fn new(frame_id: u64) -> Self {
		Self {
			header: UdpFrameHeader { protocol_version: UdpFrame::VERSION, frame_id },
			additional_headers: Default::default(),
			commands: Default::default(),
		}
	}
}


///
/// Заголовок UDP фрейма
/// - не сжимается
/// - не шифруется
/// - защищен aead
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UdpFrameHeader {
	///
	/// Версия протокола
	///
	pub protocol_version: u8,
	///
	/// Уникальный возрастающий идентификатор фрейма
	/// - игнорируем уже принятый фрейм с таким же frame_id
	/// - используется как nonce в алгоритме шифрования
	/// - должен быть уникальным, даже если это повторно отсылаемый фрейм
	///
	pub frame_id: u64,
}


///
/// Служебные данные UDP протокола
/// - не сжимается
/// - не шифруется
/// - защищены aead
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum UdpAdditionalHeader {
	ASK(AskFrameUdpItem),
	///
	/// Клиентский публичный ключ
	/// - обязательно используется в командах с клиента на сервер
	/// - необходим серверу для получения приватного ключа пользователя
	///
	UserPublicKeyC2S(UserPublicKey),
}


///
/// Прикладные данные
/// - сжимаются
/// - шифруются
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ApplicationCommand {
	Ping(String)
}

///
/// Подтверждение пакета
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AskFrameUdpItem {
	///
	/// id полученного пакета
	///
	pub ask_packet_id: u64
}