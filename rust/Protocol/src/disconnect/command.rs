use std::io::{Cursor, Error, ErrorKind};

use byteorder::{ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::frame::headers::Header;
use crate::frame::Frame;

///
/// Быстрое закрытие соединения по команде с удаленной стороны
///
#[derive(Default, Debug)]
pub struct DisconnectByCommand {
	///
	/// Соединение разорвано удаленной стороной
	///
	by_peer_reason: Option<DisconnectByCommandReason>,

	///
	/// Отправили заголовок на разрыв соединения
	///
	disconnected_by_self: bool,

	///
	/// Запрос на разрыв соединения
	///
	by_self_reason: Option<DisconnectByCommandReason>,
}
impl DisconnectByCommand {
	///
	/// Разорвать соединение с удаленной стороной
	///
	pub fn disconnect(&mut self, reason: DisconnectByCommandReason) {
		self.by_self_reason = Some(reason);
	}

	#[must_use]
	pub fn contains_self_data(&self) -> bool {
		self.by_self_reason.is_some() && !self.disconnected_by_self
	}

	pub fn build_frame(&mut self, frame: &mut Frame) {
		if let Some(reason) = self.by_self_reason {
			frame.headers.add(Header::Disconnect(DisconnectHeader(reason)));
			self.disconnected_by_self = true;
		}
	}

	pub fn on_frame_received(&mut self, frame: &Frame) {
		if let Some(header) = frame.headers.first(Header::predicate_disconnect) {
			self.by_peer_reason = Some(header.0);
		}
	}

	#[must_use]
	pub fn disconnected(&self) -> Option<DisconnectByCommandReason> {
		if self.disconnected_by_self {
			self.by_self_reason
		} else if self.by_peer_reason.is_some() {
			self.by_peer_reason
		} else {
			None
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DisconnectHeader(pub DisconnectByCommandReason);

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum DisconnectByCommandReason {
	ClientStopped = 0,
	RoomDeleted,
	MemberDeleted,
}

impl DisconnectHeader {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let reason = input.read_u8()?;
		Ok(Self(
			FromPrimitive::from_u8(reason).ok_or_else(|| Error::new(ErrorKind::InvalidData, "could not read DisconnectByCommandReason from u8".to_owned()))?,
		))
	}
	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_u8(ToPrimitive::to_u8(&self.0).ok_or_else(|| Error::new(ErrorKind::InvalidData, "could not write DisconnectByCommandReason to u8".to_owned()))?)
	}
}

#[cfg(test)]
mod tests {
	use crate::disconnect::command::{DisconnectByCommand, DisconnectByCommandReason};
	use crate::frame::headers::Header;
	use crate::frame::Frame;

	#[test]
	pub(crate) fn should_disconnect() {
		let mut self_handler = DisconnectByCommand::default();
		let mut remote_handler = DisconnectByCommand::default();

		assert!(!self_handler.contains_self_data());
		assert!(self_handler.disconnected().is_none());
		assert!(remote_handler.disconnected().is_none());

		self_handler.disconnect(DisconnectByCommandReason::ClientStopped);

		assert!(self_handler.contains_self_data());

		let mut frame = Frame::new(0, 10);
		self_handler.build_frame(&mut frame);
		remote_handler.on_frame_received(&frame);

		assert_eq!(DisconnectByCommandReason::ClientStopped, self_handler.disconnected().unwrap());
		assert_eq!(DisconnectByCommandReason::ClientStopped, remote_handler.disconnected().unwrap());
	}

	#[test]
	pub(crate) fn should_not_disconnect() {
		let mut handler = DisconnectByCommand::default();
		let mut frame = Frame::new(0, 10);
		handler.build_frame(&mut frame);
		assert!(handler.disconnected().is_none());
		assert!(matches!(frame.headers.first(Header::predicate_disconnect), None));
	}
}
