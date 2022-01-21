use std::collections::VecDeque;
use std::io::Cursor;
use std::ops::Div;
use std::time::{Duration, Instant};

use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::headers::Header;
use crate::protocol::frame::Frame;

///
/// Замеры времени round-trip
///
/// - отсылает ответ на запрос RoundTrip удаленной стороны
/// - принимает свой RoundTrip и сохраняет rtt
///
///
#[derive(Debug)]
pub struct RoundTripTime {
	start_time: Instant,
	scheduled_response: Option<RoundTripTimeHeader>,
	pub rtt: VecDeque<Duration>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct RoundTripTimeHeader {
	pub(crate) self_time: u64,
}

impl RoundTripTimeHeader {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(Self {
			self_time: input.read_variable_u64()?,
		})
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.self_time)
	}
}

impl RoundTripTime {
	pub const AVERAGE_RTT_MIN_LEN: usize = 10;

	pub fn new(start_time: &Instant) -> Self {
		Self {
			start_time: *start_time,
			scheduled_response: None,
			rtt: Default::default(),
		}
	}

	pub fn build_frame(&mut self, frame: &mut Frame, now: &Instant) {
		frame.headers.add(Header::RoundTripTimeRequest(RoundTripTimeHeader {
			self_time: now.duration_since(self.start_time).as_millis() as u64,
		}));

		match &self.scheduled_response {
			None => {}
			Some(header) => {
				frame.headers.add(Header::RoundTripTimeResponse(header.clone()));
				self.scheduled_response = None
			}
		}
	}
	///
	/// Скользящий средний для rtt
	///
	fn get_rtt(&self) -> Option<Duration> {
		if self.rtt.len() < RoundTripTime::AVERAGE_RTT_MIN_LEN {
			Option::None
		} else {
			let sum_rtt: Duration = self.rtt.iter().sum();
			let average_rtt = sum_rtt.div(self.rtt.len() as u32);
			Option::Some(average_rtt)
		}
	}

	pub fn on_frame_received(&mut self, frame: &Frame, now: &Instant) {
		// игнорируем повторно отосланные фреймы, так как они не показательны для измерения rtt
		if frame.headers.first(Header::predicate_retransmit).is_some() {
			return;
		}

		// запрос на измерение от удаленной стороны
		let request_header: Option<&RoundTripTimeHeader> = frame.headers.first(Header::predicate_round_trip_time_request);
		match request_header {
			None => {}
			Some(header) => {
				self.scheduled_response = Option::Some(header.clone());
			}
		}

		// нам пришло наше же измерение от удаленной стороны
		let response_header: Option<&RoundTripTimeHeader> = frame.headers.first(Header::predicate_round_trip_time_response);
		match response_header {
			None => {}
			Some(header) => {
				let header_time = header.self_time;
				let current_time = now.duration_since(self.start_time).as_millis() as u64;
				if current_time >= header_time {
					self.rtt.push_back(Duration::from_millis(current_time - header_time));
					if self.rtt.len() > RoundTripTime::AVERAGE_RTT_MIN_LEN {
						self.rtt.pop_front();
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};

	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::Frame;
	use crate::protocol::others::rtt::{RoundTripTime, RoundTripTimeHeader};
	use crate::protocol::reliable::retransmit::header::RetransmitHeader;

	#[test]
	///
	/// Тестируем обмен между двумя handler-ми.
	/// После обмена должно быть определено rtt.
	///
	pub fn should_calculate_rtt() {
		let mut handler_a = RoundTripTime::new(&Instant::now());
		let mut handler_b = RoundTripTime::new(&Instant::now());

		let now = Instant::now();

		let mut frame_a_b = Frame::new(1);
		handler_a.build_frame(&mut frame_a_b, &now);
		handler_b.on_frame_received(&frame_a_b, &now);

		let mut frame_b_a = Frame::new(2);
		handler_b.build_frame(&mut frame_b_a, &now);
		handler_a.on_frame_received(&frame_b_a, &now.add(Duration::from_millis(100)));

		assert!(matches!(handler_a.rtt.pop_front(), Option::Some(time) if time == Duration::from_millis(100)))
	}

	#[test]
	///
	/// Для retransmit фреймов операции получения response должны быть игнорированы
	///
	pub fn should_ignore_retransmit_frame_when_receive_response() {
		let mut handler = RoundTripTime::new(&Instant::now());
		let now = Instant::now();
		let mut frame = Frame::new(10);
		frame.headers.add(Header::Retransmit(RetransmitHeader {
			original_frame_id: 0,
			retransmit_count: 1,
		}));
		frame
			.headers
			.add(Header::RoundTripTimeResponse(RoundTripTimeHeader { self_time: 100 }));
		handler.on_frame_received(&frame, &now);
		assert!(handler.rtt.is_empty(), "{}", true);
	}

	#[test]
	///
	/// Для retransmit фреймов операции получения request должны быть игнорированы
	///
	pub fn should_ignore_retransmit_frame_when_receive_request() {
		let mut handler = RoundTripTime::new(&Instant::now());
		let now = Instant::now();

		let mut input_frame = Frame::new(10);
		input_frame.headers.add(Header::Retransmit(RetransmitHeader {
			original_frame_id: 0,
			retransmit_count: 1,
		}));
		input_frame
			.headers
			.add(Header::RoundTripTimeRequest(RoundTripTimeHeader { self_time: 100 }));
		handler.on_frame_received(&input_frame, &now);

		let mut output_frame = Frame::new(10);
		handler.build_frame(&mut output_frame, &now);

		assert!(matches!(
			output_frame.headers.first(Header::predicate_round_trip_time_response),
			Option::None
		));
	}

	///
	/// Проверяем расчет среднего rtt
	///
	/// - учитываем что для расчета среднего rtt количество измерение должно быть больше определенного значения
	///
	#[test]
	pub fn should_calculate_rtt_average() {
		let mut handler = RoundTripTime::new(&Instant::now());
		for i in 0..RoundTripTime::AVERAGE_RTT_MIN_LEN {
			let mut frame = Frame::new(10);
			frame
				.headers
				.add(Header::RoundTripTimeResponse(RoundTripTimeHeader { self_time: i as u64 }));
			let now = Instant::now().add(Duration::from_millis((i * 2) as u64));
			handler.on_frame_received(&frame, &now);
		}
		let average = handler.get_rtt();
		assert!(matches!(average, Some(average) if average==Duration::from_micros(4500)));
	}

	///
	/// Проверяем лимит на максимальный размер измерений
	///
	#[test]
	pub fn should_limit_on_length_rtt() {
		let mut handler = RoundTripTime::new(&Instant::now());
		for i in 0..2 * RoundTripTime::AVERAGE_RTT_MIN_LEN {
			let mut frame = Frame::new(10);
			frame
				.headers
				.add(Header::RoundTripTimeResponse(RoundTripTimeHeader { self_time: i as u64 }));
			let now = Instant::now().add(Duration::from_millis((i * 2) as u64));
			handler.on_frame_received(&frame, &now);
		}
		assert_eq!(handler.rtt.len(), RoundTripTime::AVERAGE_RTT_MIN_LEN);
	}
}
