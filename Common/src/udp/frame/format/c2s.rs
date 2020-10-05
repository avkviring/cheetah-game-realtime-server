use serde::{Deserialize, Serialize};

use crate::network::command::C2SCommandUnion;
use crate::network::hash::UserPublicKey;

///
/// Заголовок фрейма
/// - не шифруется и не сжимается
/// - защищен aead
///
#[derive(Serialize)]
pub struct C2SFrameHeader {
	///
	/// Версия протокола
	///
	pub protocol_version: u8,
	
	///
	/// Публичный идентификатор клиента
	/// - используется для получения приватного ключа симметричного шифра
	/// - используется для идентификации клиента
	///
	pub public_key: UserPublicKey,
	
	///
	/// Уникальный возрастающий идентификатор пакета
	/// - игнорируем уже принятый пакет с таким же packet_id
	/// - используется как nonce в алгоритме шифрования
	/// - должен быть уникальным, даже при отослать пакет повторно
	///
	pub packet_id: u64,
}

///
/// Пакеты с клиента на сервер
/// - шифруется
/// - сжимается
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum C2SFrame {
	OpenConnection(OpenConnectionC2SPacket),
	Commands(Vec<CommandC2SPacket>),
}

///
/// Первый пакет, который клиент шлет серверу
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenConnectionC2SPacket {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandC2SPacket {
	pub command: C2SCommandUnion
}