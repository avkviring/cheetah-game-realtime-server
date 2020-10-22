use serde::{Deserialize, Serialize};

use cheetah_relay_macro::EnumMatchPredicates;

use crate::commands::hash::UserPublicKey;
use crate::udp::protocol::disconnect::handler::DisconnectHeader;
use crate::udp::protocol::others::rtt::RoundTripTimeHeader;
use crate::udp::protocol::reliable::ask::header::AskFrameHeader;
use crate::udp::protocol::reliable::retransmit::RetransmitMarkHeader;

///
/// Дополнительные UDP заголовки
/// - не сжимается
/// - не шифруется
/// - защищены aead
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Headers {
	headers: Vec<Header>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, EnumMatchPredicates)]
pub enum Header {
	///
	/// Подтверждение пакета
	///
	AskFrame(AskFrameHeader),
	
	///
	/// Клиентский публичный ключ
	/// - обязательно используется в командах с клиента на сервер
	/// - необходим серверу для получения приватного ключа пользователя
	///
	UserPublicKey(UserPublicKey),
	
	///
	/// Принудительный разрыв соединения
	///
	Disconnect(DisconnectHeader),
	
	///
	/// Измерение rtt - запрос
	///
	RoundTripTimeRequest(RoundTripTimeHeader),
	
	///
	/// Измерение rtt - ответ
	///
	RoundTripTimeResponse(RoundTripTimeHeader),
	
	///
	/// Фрейм отослан повторно
	///
	RetransmitMark(RetransmitMarkHeader),
}


impl Headers {
	pub fn add(&mut self, header: Header) {
		self.headers.push(header);
	}
	
	pub fn find<T, F: FnMut(&Header) -> Option<&T>>(&self, p: F) -> Vec<&T> {
		self.headers.iter().filter_map(p).collect()
	}
	
	pub fn first<T, F: FnMut(&Header) -> Option<&T>>(&self, p: F) -> Option<&T> {
		self.headers.iter().find_map(p)
	}
}


impl Default for Headers {
	fn default() -> Self {
		Self {
			headers: Default::default()
		}
	}
}

