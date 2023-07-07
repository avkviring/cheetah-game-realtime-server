extern crate core;

use std::collections::VecDeque;
use std::fmt::Debug;
use std::time::Instant;

use crate::coniguration::ProtocolConfiguration;
use crate::disconnect::command::DisconnectByCommand;
use crate::disconnect::timeout::DisconnectByTimeout;
use crate::frame::disconnected_reason::DisconnectedReason;
use crate::frame::packets_collector::{PacketsCollector, PACKET_SIZE};
use crate::frame::segment::{Segment, SEGMENT_SIZE};
use crate::frame::{ConnectionId, Frame, FrameId};
use crate::others::keep_alive::KeepAlive;
use crate::others::rtt::RoundTripTime;
use crate::reliable::ack::AckSender;
use crate::reliable::replay_protection::FrameReplayProtection;
use crate::reliable::retransmit::Retransmitter;

pub mod codec;
pub mod collections;
pub mod coniguration;
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

pub trait InputDataHandler {
	fn on_input_data(&mut self, data: &[u8]);
}

pub trait OutputDataProducer {
	fn contains_output_data(&self) -> bool;
	fn get_output_data(&mut self, out: &mut [u8]) -> (usize, bool);
}

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
	pub next_packed_id: u64,
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
	packets_collector: PacketsCollector,
	pub configuration: ProtocolConfiguration,
}

impl<IN, OUT> Protocol<IN, OUT>
where
	IN: InputDataHandler,
	OUT: OutputDataProducer,
{
	#[must_use]
	pub fn new(input_data_handler: IN, output_data_producer: OUT, connection_id: ConnectionId, now: Instant, start_application_time: Instant, configuration: ProtocolConfiguration) -> Self {
		Self {
			next_frame_id: 1,
			next_packed_id: 0,
			disconnect_by_timeout: DisconnectByTimeout::new(now, configuration.disconnect_timeout),
			retransmitter: Retransmitter::new(configuration.disconnect_timeout),
			rtt: RoundTripTime::new(start_application_time),
			connection_id,
			input_data_handler,
			output_data_producer,
			replay_protection: Default::default(),
			ack_sender: Default::default(),
			disconnect_by_command: Default::default(),
			keep_alive: Default::default(),
			in_frame_counter: Default::default(),
			packets_collector: Default::default(),
			configuration,
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
		self.ack_sender.on_frame_received(frame, now);
		self.retransmitter.on_frame_received(frame);
		match self.replay_protection.set_and_check(frame) {
			Ok(replayed) => {
				if !replayed {
					self.disconnect_by_command.on_frame_received(frame);
					self.rtt.on_frame_received(frame, now);
					self.processing_data(frame);
				}
			}
			Err(..) => {
				tracing::error!("Replay Protection overflow")
			}
		}
	}

	fn processing_data(&mut self, frame: &Frame) {
		match self.packets_collector.on_segment(&frame.segment) {
			Ok(packet) => match packet {
				None => {}
				Some(packet) => {
					self.input_data_handler.on_input_data(packet);
				}
			},
			Err(_) => {
				tracing::error!("PacketsCollector error")
			}
		}
	}

	fn reset_protocol(&mut self, frame: &Frame, now: Instant) {
		tracing::info!("Protocol: reset protocol");
		self.connection_id = frame.connection_id;
		self.next_frame_id = 1;
		self.disconnect_by_timeout = DisconnectByTimeout::new(now, self.disconnect_by_timeout.timeout);
		self.replay_protection = Default::default();
		self.ack_sender = Default::default();
		self.retransmitter = Retransmitter::new(self.configuration.disconnect_timeout);
		self.disconnect_by_command = Default::default();
		self.keep_alive = Default::default();
		self.in_frame_counter = Default::default();
		self.packets_collector = Default::default();
	}

	///
	/// Создание фрейма для отправки
	///
	#[allow(clippy::cast_precision_loss)]
	pub fn collect_out_frames(&mut self, now: Instant, out: &mut VecDeque<Frame>) {
		match self.get_next_retransmit_frame(now) {
			None => {}
			Some(frame) => {
				out.push_back(frame);
				return;
			}
		}

		let contains_data =
			self.ack_sender.contains_self_data(now) || self.output_data_producer.contains_output_data() || self.disconnect_by_command.contains_self_data() || self.keep_alive.contains_self_data(now);

		if !contains_data {
			return;
		}

		let mut packet = [0; PACKET_SIZE];
		let (packet_size, reliability) = self.output_data_producer.get_output_data(&mut packet);
		let segments = packet[0..packet_size].chunks(SEGMENT_SIZE);

		let count_segments = segments.len();
		for (segment_number, segment_data) in segments.enumerate() {
			self.next_frame_id += 1;
			let segment = Segment::new(self.next_packed_id, count_segments as u8, segment_number as u8, segment_data);
			let mut frame = Frame::new(self.connection_id, self.next_frame_id, reliability, segment);
			self.disconnect_by_command.build_frame(&mut frame);
			self.rtt.build_frame(&mut frame, now);
			self.keep_alive.build_frame(&mut frame, now);
			self.retransmitter.build_frame(&frame, now);
			self.ack_sender.build_out_frame(&mut frame, now);
			out.push_back(frame);
		}
		self.next_packed_id += 1;
	}

	///
	/// Разорвана ли связь?
	///
	#[must_use]
	pub fn is_disconnected(&self, now: Instant) -> Option<DisconnectedReason> {
		let reason = if self.retransmitter.is_disconnected(now) {
			Some(DisconnectedReason::RetransmitOverflow)
		} else if self.disconnect_by_timeout.is_disconnected(now) {
			Some(DisconnectedReason::Timeout)
		} else {
			self.disconnect_by_command.disconnected().map(DisconnectedReason::Command)
		};
		if reason.is_some() {
			tracing::info!("Protocol: is disconnected {:?}", reason);
		}
		reason
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
	use std::time::{Duration, Instant};

	use crate::coniguration::ProtocolConfiguration;
	use crate::frame::Frame;
	use crate::frame::{ConnectionId, FrameId};
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

		fn get_output_data(&mut self, _packet: &mut [u8]) -> (usize, bool) {
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
		Frame::new(connection_id, frame_id, true, Default::default())
	}

	fn create_protocol(connection_id: ConnectionId) -> Protocol<StubDataRecvHandler, StubDataSource> {
		Protocol::<StubDataRecvHandler, StubDataSource>::new(
			Default::default(),
			Default::default(),
			connection_id,
			Instant::now(),
			Instant::now(),
			ProtocolConfiguration {
				disconnect_timeout: Duration::from_millis(100),
			},
		)
	}
}
