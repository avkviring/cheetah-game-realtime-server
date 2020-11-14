use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::protocol::{DisconnectedStatus, FrameBuilder, FrameReceivedListener};
use crate::protocol::frame::Frame;
use crate::protocol::frame::headers::Header;

///
/// Быстрое закрытие соединения по команде с удаленной стороны
///
#[derive(Default, Debug)]
pub struct DisconnectHandler {
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


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct DisconnectHeader {}

impl DisconnectHandler {
	///
	/// Разорвать соединение с удаленной стороной
	///
	pub fn disconnect(&mut self) {
		self.disconnecting_by_self_request = true;
	}
}

impl FrameBuilder for DisconnectHandler {
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

impl FrameReceivedListener for DisconnectHandler {
	fn on_frame_received(&mut self, frame: &Frame, _: &Instant) {
		let headers: Option<&DisconnectHeader> = frame.headers.first(Header::predicate_disconnect);
		self.disconnected_by_peer = headers.is_some();
	}
}

impl DisconnectedStatus for DisconnectHandler {
	fn disconnected(&self, _: &Instant) -> bool {
		self.disconnected_by_peer || self.disconnected_by_self
	}
}

#[cfg(test)]
mod tests {
	use std::time::Instant;
	
	use crate::protocol::{DisconnectedStatus, FrameBuilder, FrameReceivedListener};
	use crate::protocol::disconnect::handler::DisconnectHandler;
	use crate::protocol::frame::Frame;
	use crate::protocol::frame::headers::Header;
	
	#[test]
	pub fn should_disconnect() {
		let now = Instant::now();
		let mut self_handler = DisconnectHandler::default();
		let mut remote_handler = DisconnectHandler::default();
		
		assert_eq!(self_handler.contains_self_data(&now), false);
		assert_eq!(self_handler.disconnected(&now), false);
		assert_eq!(remote_handler.disconnected(&now), false);
		
		self_handler.disconnect();
		
		assert_eq!(self_handler.contains_self_data(&now), true);
		
		let mut frame = Frame::new(10);
		self_handler.build_frame(&mut frame, &now);
		remote_handler.on_frame_received(&frame, &now);
		
		assert_eq!(self_handler.disconnected(&now), true);
		assert_eq!(remote_handler.disconnected(&now), true);
	}
	
	#[test]
	pub fn should_not_disconnect() {
		let now = Instant::now();
		let mut handler = DisconnectHandler::default();
		let mut frame = Frame::new(10);
		handler.build_frame(&mut frame, &now);
		assert_eq!(handler.disconnected(&now), false);
		assert!(matches!(frame.headers.first(Header::predicate_disconnect), Option::None));
	}
}