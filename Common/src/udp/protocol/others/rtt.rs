use std::any::Any;
use std::ops::Sub;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::udp::protocol::{FrameBuilder, FrameReceivedListener};
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::frame::headers::Header;

///
/// Замеры времени round-trip
///
/// - отсылает ответ на запрос RoundTrip удаленной стороны
/// - принимает свой RoundTrip и сохраняет rtt
///
pub struct RoundTripTimeHandler {
	start_time: Instant,
	scheduled_response: Option<RoundTripTimeHeader>,
	pub rtt: Option<Duration>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct RoundTripTimeHeader {
	self_time: u64
}


impl Default for RoundTripTimeHandler {
	fn default() -> Self {
		Self {
			start_time: Instant::now(),
			scheduled_response: None,
			rtt: None,
		}
	}
}

impl FrameReceivedListener for RoundTripTimeHandler {
	fn on_frame_received(&mut self, frame: &Frame, now: &Instant) {
		
		// игнорируем повторно отосланные фреймы, так как они не показательны для измерения rtt
		if frame.headers.first(Header::predicate_RetransmitMark).is_some() {
			return;
		}
		
		// запрос на измерение от удаленной стороны
		let request_header: Option<&RoundTripTimeHeader> = frame.headers.first(Header::predicate_RoundTripTimeRequest);
		match request_header {
			None => {}
			Some(header) => {
				self.scheduled_response = Option::Some(header.clone());
			}
		}
		
		// нам пришло наше же измерение от удаленной стороны
		let response_header: Option<&RoundTripTimeHeader> = frame.headers.first(Header::predicate_RoundTripTimeResponse);
		match response_header {
			None => {}
			Some(header) => {
				let header_time = header.self_time;
				let current_time = now.duration_since(self.start_time).as_millis() as u64;
				if current_time >= header_time {
					self.rtt = Option::Some(Duration::from_millis(current_time - header_time));
				}
			}
		}
	}
}


impl FrameBuilder for RoundTripTimeHandler {
	fn contains_self_data(&self, now: &Instant) -> bool {
		false
	}
	
	fn build_frame(&mut self, frame: &mut Frame, now: &Instant) {
		frame.headers.add(Header::RoundTripTimeRequest(RoundTripTimeHeader {
			self_time: now.duration_since(self.start_time).as_millis() as u64
		}));
		
		match &self.scheduled_response {
			None => {}
			Some(header) => {
				frame.headers.add(Header::RoundTripTimeResponse(header.clone()));
				self.scheduled_response = None
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};
	
	use crate::udp::protocol::{FrameBuilder, FrameReceivedListener};
	use crate::udp::protocol::frame::Frame;
	use crate::udp::protocol::frame::headers::Header;
	use crate::udp::protocol::others::rtt::{RoundTripTimeHandler, RoundTripTimeHeader};
	use crate::udp::protocol::reliable::retransmit::RetransmitMarkHeader;
	
	#[test]
	///
	/// Тестируем обмен между двумя handler-ми.
	/// После обмена должно быть определено rtt.
	///
	pub fn should_calculate_rtt() {
		let mut handler_a = RoundTripTimeHandler::default();
		let mut handler_b = RoundTripTimeHandler::default();
		
		let now = Instant::now();
		
		let mut frame_a_b = Frame::new(1);
		handler_a.build_frame(&mut frame_a_b, &now);
		handler_b.on_frame_received(&frame_a_b, &now);
		
		let mut frame_b_a = Frame::new(2);
		handler_b.build_frame(&mut frame_b_a, &now);
		handler_a.on_frame_received(&frame_b_a, &now.add(Duration::from_millis(100)));
		
		assert!(matches!(handler_a.rtt, Option::Some(time) if time == Duration::from_millis(100)))
	}
	
	#[test]
	///
	/// Для retransmit фреймов операции получения response должны быть игнорированы
	///
	pub fn should_ignore_retransmit_frame_when_receive_response() {
		let mut handler = RoundTripTimeHandler::default();
		let now = Instant::now();
		let mut frame = Frame::new(10);
		frame.headers.add(Header::RetransmitMark(RetransmitMarkHeader { retransmit_count: 1 }));
		frame.headers.add(Header::RoundTripTimeResponse(RoundTripTimeHeader { self_time: 100 }));
		handler.on_frame_received(&frame, &now);
		assert!(matches!(handler.rtt, Option::None));
	}
	
	#[test]
	///
	/// Для retransmit фреймов операции получения request должны быть игнорированы
	///
	pub fn should_ignore_retransmit_frame_when_receive_request() {
		let mut handler = RoundTripTimeHandler::default();
		let now = Instant::now();
		
		let mut input_frame = Frame::new(10);
		input_frame.headers.add(Header::RetransmitMark(RetransmitMarkHeader { retransmit_count: 1 }));
		input_frame.headers.add(Header::RoundTripTimeRequest(RoundTripTimeHeader { self_time: 100 }));
		handler.on_frame_received(&input_frame, &now);
		
		let mut output_frame = Frame::new(10);
		handler.build_frame(&mut output_frame, &now);
		
		assert!(matches!(output_frame.headers.first(Header::predicate_RoundTripTimeResponse), Option::None));
	}
}
