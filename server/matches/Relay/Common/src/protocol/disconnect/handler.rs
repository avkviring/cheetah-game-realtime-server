use crate::protocol::frame::headers::Header;
use crate::protocol::frame::Frame;

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

	pub fn contains_self_data(&self) -> bool {
		self.disconnecting_by_self_request && !self.disconnected_by_self
	}

	pub fn build_frame(&mut self, frame: &mut Frame) {
		if self.disconnecting_by_self_request {
			frame.headers.add(Header::Disconnect(DisconnectHeader::default()));
			self.disconnected_by_self = true;
		}
	}

	pub fn on_frame_received(&mut self, frame: &Frame) {
		let headers: Option<&DisconnectHeader> = frame.headers.first(Header::predicate_disconnect);
		self.disconnected_by_peer = headers.is_some();
	}

	pub fn disconnected(&self) -> bool {
		self.disconnected_by_peer || self.disconnected_by_self
	}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DisconnectHeader {}

#[cfg(test)]
mod tests {
	use std::time::Instant;

	use crate::protocol::disconnect::handler::DisconnectCommandHandler;
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::Frame;

	#[test]
	pub fn should_disconnect() {
		let mut self_handler = DisconnectCommandHandler::default();
		let mut remote_handler = DisconnectCommandHandler::default();

		assert!(!self_handler.contains_self_data());
		assert!(!self_handler.disconnected());
		assert!(!remote_handler.disconnected());

		self_handler.disconnect();

		assert!(self_handler.contains_self_data());

		let mut frame = Frame::new(10);
		self_handler.build_frame(&mut frame);
		remote_handler.on_frame_received(&frame);

		assert!(self_handler.disconnected());
		assert!(remote_handler.disconnected());
	}

	#[test]
	pub fn should_not_disconnect() {
		let mut handler = DisconnectCommandHandler::default();
		let mut frame = Frame::new(10);
		handler.build_frame(&mut frame);
		assert!(!handler.disconnected());
		assert!(matches!(frame.headers.first(Header::predicate_disconnect), Option::None));
	}
}
