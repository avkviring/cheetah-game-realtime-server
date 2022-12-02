use crate::protocol::frame::headers::Header;
use crate::protocol::frame::input::InFrame;
use crate::protocol::frame::output::OutFrame;

///
/// Быстрое закрытие соединения по команде с удаленной стороны
///
#[derive(Default, Debug)]
pub struct DisconnectByCommand {
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
impl DisconnectByCommand {
	///
	/// Разорвать соединение с удаленной стороной
	///
	pub fn disconnect(&mut self) {
		self.disconnecting_by_self_request = true;
	}

	#[must_use]
	pub fn contains_self_data(&self) -> bool {
		self.disconnecting_by_self_request && !self.disconnected_by_self
	}

	pub fn build_frame(&mut self, frame: &mut OutFrame) {
		if self.disconnecting_by_self_request {
			frame.headers.add(Header::Disconnect(DisconnectHeader::default()));
			self.disconnected_by_self = true;
		}
	}

	pub fn on_frame_received(&mut self, frame: &InFrame) {
		let headers: Option<&DisconnectHeader> = frame.headers.first(Header::predicate_disconnect);
		self.disconnected_by_peer = headers.is_some();
	}

	#[must_use]
	pub fn disconnected(&self) -> bool {
		self.disconnected_by_peer || self.disconnected_by_self
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DisconnectHeader;

#[cfg(test)]
mod tests {
	use crate::protocol::disconnect::command::DisconnectByCommand;
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::input::InFrame;
	use crate::protocol::frame::output::OutFrame;

	#[test]
	pub(crate) fn should_disconnect() {
		let mut self_handler = DisconnectByCommand::default();
		let mut remote_handler = DisconnectByCommand::default();

		assert!(!self_handler.contains_self_data());
		assert!(!self_handler.disconnected());
		assert!(!remote_handler.disconnected());

		self_handler.disconnect();

		assert!(self_handler.contains_self_data());

		let mut frame = OutFrame::new(10);
		self_handler.build_frame(&mut frame);
		remote_handler.on_frame_received(&InFrame::new(frame.frame_id, frame.headers, Default::default()));

		assert!(self_handler.disconnected());
		assert!(remote_handler.disconnected());
	}

	#[test]
	pub(crate) fn should_not_disconnect() {
		let mut handler = DisconnectByCommand::default();
		let mut frame = OutFrame::new(10);
		handler.build_frame(&mut frame);
		assert!(!handler.disconnected());
		assert!(matches!(frame.headers.first(Header::predicate_disconnect), None));
	}
}
