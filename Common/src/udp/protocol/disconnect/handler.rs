use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::udp::protocol::{DisconnectedStatus, FrameReceivedListener, FrameBuilder};
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::frame::headers::Header;

///
/// Быстрое закрытие соединения по команде с удаленной стороны
///
#[derive(Default)]
pub struct DisconnectHandler {
	///
	/// Соединение разорвано удаленной стороной
	///
	disconnected_by_peer: bool,
	
	///
	/// Запрос на разрыв соединения
	///
	disconnecting_by_self: bool,
	
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
		self.disconnecting_by_self = true;
	}
}

impl FrameBuilder for DisconnectHandler {
	fn contains_self_data(&self, now: &Instant) -> bool {
		self.disconnecting_by_self
	}
	
	fn build_frame(&mut self, frame: &mut Frame, now: &Instant) {
		frame.headers.add(Header::Disconnect(DisconnectHeader::default()));
		self.disconnected_by_self = true;
	}
}

impl FrameReceivedListener for DisconnectHandler {
	fn on_frame_received(&mut self, frame: &Frame, now: &Instant) {
		let headers: Option<&DisconnectHeader> = frame.headers.first(Header::predicate_Disconnect);
		self.disconnected_by_peer = headers.is_some();
	}
}

impl DisconnectedStatus for DisconnectHandler {
	fn disconnected(&mut self, now: &Instant) -> bool {
		self.disconnected_by_peer || self.disconnected_by_self
	}
}

#[cfg(test)]
mod tests {
	use std::time::Instant;
	
	use crate::udp::protocol::{DisconnectedStatus, FrameReceivedListener, FrameBuilder};
	use crate::udp::protocol::disconnect::handler::DisconnectHandler;
	use crate::udp::protocol::frame::Frame;
	
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
}