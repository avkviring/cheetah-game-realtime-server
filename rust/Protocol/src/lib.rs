extern crate core;

use std::fmt::Debug;
use std::time::Instant;

use prometheus::local::{LocalHistogram, LocalIntCounter};

use crate::disconnect::command::DisconnectByCommand;
use crate::disconnect::timeout::DisconnectByTimeout;
use crate::frame::disconnected_reason::DisconnectedReason;
use crate::frame::{ConnectionId, Frame, FrameId, FRAME_BODY_CAPACITY};
use crate::others::keep_alive::KeepAlive;
use crate::others::rtt::RoundTripTime;
use crate::reliable::ack::AckSender;
use crate::reliable::replay_protection::FrameReplayProtection;
use crate::reliable::retransmit::Retransmitter;

pub mod codec;
pub mod collections;
pub mod disconnect;
pub mod frame;
pub mod others;
pub mod reliable;
pub mod trace;
pub mod trace_collector;

pub type RoomMemberId = u16;
pub type RoomId = u64;

///
/// Примерное количество фреймов в секунду на одного peer
/// - необходимо для расчетов размеров структур
/// - в точности нет необходимости, но не должно отличаться на порядки
///
pub const MAX_FRAME_PER_SECONDS: usize = 120;

///
/// Если от peer не будет фреймов за данное время - считаем что соединение разорвано
///
pub const DISCONNECT_TIMEOUT_IN_SECONDS: usize = 60;

pub const NOT_EXIST_FRAME_ID: FrameId = 0;

///
/// Реализация игрового протокола, поверх ненадежного канала доставки данных (например, через UDP)
///
/// - логическая часть, без сети и сериализации
/// - надежная доставка
/// - защита от повторов
///
#[derive(Debug)]
pub struct Protocol<IN, OUT>
where
	IN: InputDataHandler,
	OUT: OutputDataProducer,
{
	pub connection_id: ConnectionId,
	pub next_frame_id: u64,
	pub replay_protection: FrameReplayProtection,
	pub ack_sender: AckSender,
	pub retransmitter: Retransmitter,
	pub disconnect_by_timeout: DisconnectByTimeout,
	pub disconnect_by_command: DisconnectByCommand,
	pub input_data_handler: IN,
	pub output_data_producer: OUT,
	pub rtt: RoundTripTime,
	pub keep_alive: KeepAlive,
	pub in_frame_counter: u64,
	ack_sent_histogram: LocalHistogram,
	retransmit_counter: LocalIntCounter,
}

pub trait InputDataHandler {
	fn on_input_data(&mut self, data: &[u8]);
}

pub trait OutputDataProducer {
	fn contains_output_data(&self) -> bool;
	fn get_output_data(&mut self, buffer: &mut [u8; FRAME_BODY_CAPACITY]) -> (usize, bool);
}

impl<IN, OUT> Protocol<IN, OUT>
where
	IN: InputDataHandler,
	OUT: OutputDataProducer,
{
	#[must_use]
	pub fn new(
		input_data_handler: IN,
		output_data_producer: OUT,
		connection_id: ConnectionId,
		now: Instant,
		start_application_time: Instant,
		retransmit_counter: LocalIntCounter,
		ack_sent_histogram: LocalHistogram,
	) -> Self {
		Self {
			connection_id,
			input_data_handler,
			output_data_producer,
			ack_sent_histogram,
			next_frame_id: 1,
			disconnect_by_timeout: DisconnectByTimeout::new(now),
			replay_protection: Default::default(),
			ack_sender: Default::default(),
			retransmitter: Retransmitter::new(retransmit_counter.clone()),
			disconnect_by_command: Default::default(),
			rtt: RoundTripTime::new(start_application_time),
			keep_alive: Default::default(),
			in_frame_counter: Default::default(),
			retransmit_counter,
		}
	}

	///
	/// Обработка входящего фрейма
	///
	pub fn on_frame_received(&mut self, frame: &Frame, now: Instant) {
		// у другой стороны уже новый идентификатор соединения
		if frame.connection_id > self.connection_id {
			self.reset_protocol(frame, now);
		}

		// игнорируем все входящие фреймы не с текущем идентификатором соединения
		if frame.connection_id != self.connection_id {
			return;
		}

		self.in_frame_counter += 1;
		self.disconnect_by_timeout.on_frame_received(now);
		self.retransmitter.on_frame_received(frame, now);
		self.ack_sender.on_frame_received(frame, now);
		match self.replay_protection.set_and_check(frame) {
			Ok(replayed) => {
				if !replayed {
					self.disconnect_by_command.on_frame_received(frame);
					self.rtt.on_frame_received(frame, now);
					self.input_data_handler.on_input_data(frame.get_body());
				}
			}
			Err(..) => {
				tracing::error!("Replay Protection overflow")
			}
		}
	}

	fn reset_protocol(&mut self, frame: &Frame, now: Instant) {
		self.connection_id = frame.connection_id;
		self.next_frame_id = 1;
		self.disconnect_by_timeout = DisconnectByTimeout::new(now);
		self.replay_protection = Default::default();
		self.ack_sender = Default::default();
		self.retransmitter = Retransmitter::new(self.retransmit_counter.clone());
		self.disconnect_by_command = Default::default();
		self.keep_alive = Default::default();
		self.in_frame_counter = Default::default();
	}

	///
	/// Создание фрейма для отправки
	///
	#[allow(clippy::cast_precision_loss)]
	pub fn build_next_frame(&mut self, now: Instant) -> Option<Frame> {
		match self.get_next_retransmit_frame(now) {
			None => {}
			Some(frame) => {
				return Some(frame);
			}
		}

		let contains_data =
			self.ack_sender.contains_self_data(now) || self.output_data_producer.contains_output_data() || self.disconnect_by_command.contains_self_data() || self.keep_alive.contains_self_data(now);

		contains_data.then(|| {
			let mut frame = Frame::new(self.connection_id, self.next_frame_id);
			self.next_frame_id += 1;
			let (size, reliability) = self.output_data_producer.get_output_data(&mut frame.body);
			frame.body_size = size;
			frame.reliability = reliability;
			let acked_task_count = self.ack_sender.build_out_frame(&mut frame, now);
			self.ack_sent_histogram.observe(acked_task_count as f64);
			self.disconnect_by_command.build_frame(&mut frame);
			self.rtt.build_frame(&mut frame, now);
			self.keep_alive.build_frame(&mut frame, now);
			self.retransmitter.build_frame(&frame, now);
			frame
		})
	}

	///
	/// Разорвана ли связь?
	///
	#[must_use]
	pub fn is_disconnected(&self, now: Instant) -> Option<DisconnectedReason> {
		if let Err(reason) = self.retransmitter.disconnected(now) {
			Some(reason)
		} else if self.disconnect_by_timeout.disconnected(now) {
			Some(DisconnectedReason::ByTimeout)
		} else {
			self.disconnect_by_command.disconnected().map(DisconnectedReason::ByCommand)
		}
	}

	///
	/// Установлено ли соединения?
	///
	#[must_use]
	pub fn is_connected(&self, now: Instant) -> bool {
		self.in_frame_counter > 0 && self.is_disconnected(now).is_none()
	}

	pub fn get_next_retransmit_frame(&mut self, now: Instant) -> Option<Frame> {
		let next_frame_id = self.next_frame_id + 1;
		match self.retransmitter.get_retransmit_frame(now, next_frame_id) {
			None => None,
			Some(frame) => {
				self.next_frame_id = next_frame_id;
				Some(frame)
			}
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::time::Instant;

	use prometheus::{Histogram, HistogramOpts, IntCounter};

	use crate::frame::{ConnectionId, FrameId};
	use crate::frame::{Frame, FRAME_BODY_CAPACITY};
	use crate::{InputDataHandler, OutputDataProducer, Protocol};

	#[derive(Default)]
	struct StubDataRecvHandler {
		on_recv_count: usize,
	}

	impl InputDataHandler for StubDataRecvHandler {
		fn on_input_data(&mut self, _data: &[u8]) {
			self.on_recv_count += 1;
		}
	}

	#[derive(Default)]
	struct StubDataSource {}

	impl OutputDataProducer for StubDataSource {
		fn contains_output_data(&self) -> bool {
			true
		}

		fn get_output_data(&mut self, _buffer: &mut [u8; FRAME_BODY_CAPACITY]) -> (usize, bool) {
			(0, false)
		}
	}

	#[test]
	fn should_dont_apply_commands_from_frame_id_with_different_connection_id() {
		let mut protocol = create_protocol(5);
		protocol.on_frame_received(&create_frame(1, 1), Instant::now());
		assert_eq!(protocol.input_data_handler.on_recv_count, 0);
	}

	fn create_frame(connection_id: ConnectionId, frame_id: FrameId) -> Frame {
		Frame::new(connection_id, frame_id)
	}

	fn create_protocol(connection_id: ConnectionId) -> Protocol<StubDataRecvHandler, StubDataSource> {
		let counter = IntCounter::new("name", "help").unwrap().local();
		let histogram = Histogram::with_opts(HistogramOpts::new("name", "help")).unwrap().local();
		return Protocol::<StubDataRecvHandler, StubDataSource>::new(Default::default(), Default::default(), connection_id, Instant::now(), Instant::now(), counter, histogram);
	}
}
