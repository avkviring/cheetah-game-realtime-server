use cheetah_matches_relay_macro::EnumMatchPredicates;

use crate::protocol::disconnect::handler::DisconnectHeader;
use crate::protocol::others::rtt::RoundTripTimeHeader;
use crate::protocol::others::user_id::MemberAndRoomId;
use crate::protocol::reliable::ack::header::AckHeader;
use crate::protocol::reliable::retransmit::RetransmitHeader;

///
/// Дополнительные UDP заголовки
/// - не сжимается
/// - не шифруется
/// - защищены aead
///
#[derive(Debug, PartialEq, Clone)]
pub struct Headers {
	pub(crate) headers: Vec<Header>,
}

#[derive(Debug, PartialEq, Clone, EnumMatchPredicates)]
pub enum Header {
	///
	/// Идентификатор комнаты и клиента
	/// Посылается от клиента к серверу
	///
	MemberAndRoomId(MemberAndRoomId),

	///
	/// Подтверждение пакета
	///
	Ack(AckHeader),

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
	Retransmit(RetransmitHeader),

	///
	/// Приветственный пакет
	///
	Hello,
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
			headers: Default::default(),
		}
	}
}
