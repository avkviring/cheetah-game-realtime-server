use cheetah_matches_realtime_macro::EnumMatchPredicates;

use crate::protocol::disconnect::command::DisconnectHeader;
use crate::protocol::others::rtt::RoundTripTimeHeader;
use crate::protocol::others::user_id::MemberAndRoomId;
use crate::protocol::reliable::ack::header::AckHeader;
use crate::protocol::reliable::retransmit::header::RetransmitHeader;

pub type HeaderVec<T> = heapless::Vec<T, 10>;
///
/// Дополнительные UDP заголовки
/// - не сжимается
/// - не шифруется
/// - защищены aead
///
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Headers {
	pub(crate) headers: HeaderVec<Header>,
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
	pub fn is_full(&self) -> bool {
		tracing::info!(
			"is_full  {:?} {:?}",
			self.headers.capacity(),
			self.headers.len()
		);
		self.headers.capacity() == self.headers.len()
	}

	pub fn add(&mut self, header: Header) {
		if self.headers.push(header).is_err() {
			panic!("Headers vector overflow {:?}", self.headers)
		}
	}

	pub fn find<T, F: FnMut(&Header) -> Option<&T>>(&self, p: F) -> HeaderVec<&T> {
		self.headers.iter().filter_map(p).collect()
	}

	pub fn first<T, F: FnMut(&Header) -> Option<&T>>(&self, p: F) -> Option<&T> {
		self.headers.iter().find_map(p)
	}
}
