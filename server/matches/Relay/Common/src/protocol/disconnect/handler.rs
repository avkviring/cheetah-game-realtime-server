use std::time::Instant;

use crate::protocol::frame::headers::Header;
use crate::protocol::frame::Frame;
use crate::protocol::{DisconnectedStatus, FrameBuilder, FrameReceivedListener};

///
/// Быстрое закрытие соединения по команде с удаленной стороны
///
#[derive(Default, Debug)]
pub struct DisconnectCommandHandler {
	///
	/// Соединение разорвано удаленной стороной
	///
	disconnected_by_peer: bool,

	///
	/// Запрос на разрыв соединения
	///
	disconnecting_by_self_request: bool,

	///
	/// Отправили заголовок на разрыв соединения
	///
	disconnected_by_self: bool,
}
impl DisconnectCommandHandler {
	///
	/// Разорвать соединение с удаленной стороной
	///
	pub fn disconnect(&mut self) {
		self.disconnecting_by_self_request = true;
	}
}

impl FrameBuilder for DisconnectCommandHandler {
	fn contains_self_data(&self, _: &Instant) -> bool {
		self.disconnecting_by_self_request && !self.disconnected_by_self
	}

	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		if self.disconnecting_by_self_request {
			frame.headers.add(Header::Disconnect(DisconnectHeader::default()));
			self.disconnected_by_self = true;
		}
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DisconnectHeader {}

impl FrameReceivedListener for DisconnectCommandHandler {
	fn on_frame_received(&mut self, frame: &Frame, _: &Instant) {
		let headers: Option<&DisconnectHeader> = frame.headers.first(Header::predicate_disconnect);
		self.disconnected_by_peer = headers.is_some();
	}
}

impl DisconnectedStatus for DisconnectCommandHandler {
	fn disconnected(&self, _: &Instant) -> bool {
		self.disconnected_by_peer || self.disconnected_by_self
	}
}

#[cfg(test)]
mod tests {
	use std::time::Instant;

	use crate::protocol::disconnect::handler::DisconnectCommandHandler;
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::Frame;
	use crate::protocol::{DisconnectedStatus, FrameBuilder, FrameReceivedListener};

	#[test]
	pub fn should_disconnect() {
		let now = Instant::now();
		let mut self_handler = DisconnectCommandHandler::default();
		let mut remote_handler = DisconnectCommandHandler::default();

		assert!(!self_handler.contains_self_data(&now));
		assert!(!self_handler.disconnected(&now));
		assert!(!remote_handler.disconnected(&now));

		self_handler.disconnect();

		assert!(self_handler.contains_self_data(&now));

		let mut frame = Frame::new(10);
		self_handler.build_frame(&mut frame, &now);
		remote_handler.on_frame_received(&frame, &now);

		assert!(self_handler.disconnected(&now));
		assert!(remote_handler.disconnected(&now));
	}

	#[test]
	pub fn should_not_disconnect() {
		let now = Instant::now();
		let mut handler = DisconnectCommandHandler::default();
		let mut frame = Frame::new(10);
		handler.build_frame(&mut frame, &now);
		assert!(!handler.disconnected(&now));
		assert!(matches!(frame.headers.first(Header::predicate_disconnect), Option::None));
	}
}
