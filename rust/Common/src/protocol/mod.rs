use std::fmt::Debug;
use std::time::Instant;

use prometheus::local::{LocalHistogram, LocalIntCounter};

use crate::network::channel::DisconnectedReason;
use crate::protocol::commands::input::InCommandsCollector;
use crate::protocol::commands::output::OutCommandsCollector;
use crate::protocol::disconnect::command::DisconnectByCommand;
use crate::protocol::disconnect::timeout::DisconnectByTimeout;
use crate::protocol::frame::input::InFrame;
use crate::protocol::frame::output::OutFrame;
use crate::protocol::frame::{ConnectionId, FrameId};
use crate::protocol::others::keep_alive::KeepAlive;
use crate::protocol::others::rtt::RoundTripTime;
use crate::protocol::reliable::ack::AckSender;
use crate::protocol::reliable::replay_protection::FrameReplayProtection;
use crate::protocol::reliable::retransmit::Retransmitter;

pub mod codec;
pub mod commands;
pub mod disconnect;
pub mod frame;
pub mod others;
pub mod reliable;

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
pub struct Protocol {
	pub connection_id: ConnectionId,
	pub next_frame_id: u64,
	pub replay_protection: FrameReplayProtection,
	pub ack_sender: AckSender,
	pub retransmitter: Retransmitter,
	pub disconnect_by_timeout: DisconnectByTimeout,
	pub disconnect_by_command: DisconnectByCommand,
	pub in_commands_collector: InCommandsCollector,
	pub out_commands_collector: OutCommandsCollector,
	pub rtt: RoundTripTime,
	pub keep_alive: KeepAlive,
	pub in_frame_counter: u64,
	ack_sent_histogram: LocalHistogram,
	retransmit_counter: LocalIntCounter,
}

impl Protocol {
	#[must_use]
	pub fn new(connection_id: ConnectionId, now: Instant, start_application_time: Instant, retransmit_counter: LocalIntCounter, ack_sent_histogram: LocalHistogram) -> Self {
		Self {
			connection_id,
			next_frame_id: 1,
			disconnect_by_timeout: DisconnectByTimeout::new(now),
			replay_protection: Default::default(),
			ack_sender: Default::default(),
			in_commands_collector: Default::default(),
			out_commands_collector: Default::default(),
			retransmitter: Retransmitter::new(connection_id, retransmit_counter.clone()),
			disconnect_by_command: Default::default(),
			rtt: RoundTripTime::new(start_application_time),
			keep_alive: Default::default(),
			in_frame_counter: Default::default(),
			ack_sent_histogram,
			retransmit_counter,
		}
	}

	///
	/// Обработка входящего фрейма
	///
	pub fn on_frame_received(&mut self, frame: &InFrame, now: Instant) {
		// у другой стороны уже новый идентификатор соединения
		if frame.connection_id > self.connection_id {
			*self = Protocol::new(frame.connection_id, now, now, self.retransmit_counter.clone(), self.ack_sent_histogram.clone());
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
					self.in_commands_collector.collect(frame);
				}
			}
			Err(..) => {
				tracing::error!("Replay Protection overflow")
			}
		}
	}

	///
	/// Создание фрейма для отправки
	///
	#[allow(clippy::cast_precision_loss)]
	pub fn build_next_frame(&mut self, now: Instant) -> Option<OutFrame> {
		match self.get_next_retransmit_frame(now) {
			None => {}
			Some(frame) => {
				return Some(frame);
			}
		}

		let contains_data =
			self.ack_sender.contains_self_data(now) || self.out_commands_collector.contains_self_data() || self.disconnect_by_command.contains_self_data() || self.keep_alive.contains_self_data(now);

		contains_data.then(|| {
			let mut frame = OutFrame::new(self.connection_id, self.next_frame_id);
			self.next_frame_id += 1;

			let acked_task_count = self.ack_sender.build_out_frame(&mut frame, now);
			self.ack_sent_histogram.observe(acked_task_count as f64);

			self.out_commands_collector.build_frame(&mut frame);
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

	pub fn get_next_retransmit_frame(&mut self, now: Instant) -> Option<OutFrame> {
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

	use crate::commands::c2s::C2SCommand;
	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithReliabilityGuarantees};
	use crate::protocol::frame::channel::ReliabilityGuaranteesChannel;
	use crate::protocol::frame::input::InFrame;
	use crate::protocol::frame::{ConnectionId, FrameId};
	use crate::protocol::Protocol;

	#[test]
	fn should_dont_apply_commands_from_frame_id_with_different_connection_id() {
		let mut protocol = create_protocol(5);
		protocol.on_frame_received(&create_frame(1, 1), Instant::now());
		assert_eq!(protocol.in_commands_collector.get_ready_commands().len(), 0);
	}

	#[test]
	fn should_switch_protocol_from_new_connection_id() {
		let mut protocol = create_protocol(1);
		protocol.on_frame_received(&create_frame(1, 1), Instant::now());
		protocol.on_frame_received(&create_frame(1, 2), Instant::now());
		protocol.on_frame_received(&create_frame(2, 3), Instant::now());
		assert_eq!(protocol.in_commands_collector.get_ready_commands().len(), 1);
	}

	fn create_frame(connection_id: ConnectionId, frame_id: FrameId) -> InFrame {
		InFrame::new(
			connection_id,
			frame_id,
			Default::default(),
			vec![CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::UnreliableUnordered,
				commands: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			}],
		)
	}

	fn create_protocol(connection_id: ConnectionId) -> Protocol {
		let counter = IntCounter::new("name", "help").unwrap().local();
		let histogram = Histogram::with_opts(HistogramOpts::new("name", "help")).unwrap().local();
		return Protocol::new(connection_id, Instant::now(), Instant::now(), counter, histogram);
	}
}
