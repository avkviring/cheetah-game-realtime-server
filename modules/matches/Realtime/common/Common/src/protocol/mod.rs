use std::fmt::Debug;
use std::time::Instant;

use crate::network::client::DisconnectedReason;
use crate::protocol::commands::input::InCommandsCollector;
use crate::protocol::commands::output::OutCommandsCollector;
use crate::protocol::disconnect::command::DisconnectByCommand;
use crate::protocol::disconnect::timeout::DisconnectByTimeout;
use crate::protocol::frame::input::InFrame;
use crate::protocol::frame::output::OutFrame;
use crate::protocol::frame::FrameId;
use crate::protocol::others::keep_alive::KeepAlive;
use crate::protocol::others::rtt::RoundTripTime;
use crate::protocol::reliable::ack::AckSender;
use crate::protocol::reliable::replay_protection::FrameReplayProtection;
use crate::protocol::reliable::retransmit::Retransmit;

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
pub const MAX_FRAME_PER_SECONDS: usize = 60;

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
	pub next_frame_id: u64,
	pub replay_protection: FrameReplayProtection,
	pub ack_sender: AckSender,
	pub retransmitter: Retransmit,
	pub disconnect_by_timeout: DisconnectByTimeout,
	pub disconnect_by_command: DisconnectByCommand,
	pub in_commands_collector: InCommandsCollector,
	pub out_commands_collector: OutCommandsCollector,
	pub rtt: RoundTripTime,
	pub keep_alive: KeepAlive,
	pub in_frame_counter: u64,
}

impl Protocol {
	#[must_use]
	pub fn new(now: Instant, start_application_time: Instant) -> Self {
		Self {
			next_frame_id: 1,
			disconnect_by_timeout: DisconnectByTimeout::new(now),
			replay_protection: Default::default(),
			ack_sender: Default::default(),
			in_commands_collector: Default::default(),
			out_commands_collector: Default::default(),
			retransmitter: Default::default(),
			disconnect_by_command: Default::default(),
			rtt: RoundTripTime::new(start_application_time),
			keep_alive: Default::default(),
			in_frame_counter: Default::default(),
		}
	}

	///
	/// Обработка входящего фрейма
	///
	pub fn on_frame_received(&mut self, frame: &InFrame, now: Instant) {
		self.in_frame_counter += 1;
		self.disconnect_by_timeout.on_frame_received(now);
		self.retransmitter.on_frame_received(frame, now);
		if let Ok(replayed) = self.replay_protection.set_and_check(frame) {
			if !replayed {
				self.disconnect_by_command.on_frame_received(frame);
				self.ack_sender.on_frame_received(frame, now);
				self.rtt.on_frame_received(frame, now);
				self.in_commands_collector.collect(frame);
			}
		}
	}

	///
	/// Создание фрейма для отправки
	///
	pub fn build_next_frame(&mut self, now: Instant) -> Option<OutFrame> {
		match self.get_next_retransmit_frame(now) {
			None => {}
			Some(frame) => {
				return Some(frame);
			}
		}

		let contains_data = self.ack_sender.contains_self_data(now)
			|| self.out_commands_collector.contains_self_data()
			|| self.disconnect_by_command.contains_self_data()
			|| self.keep_alive.contains_self_data(now);

		contains_data.then(|| {
			let mut frame = OutFrame::new(self.next_frame_id);
			self.next_frame_id += 1;

			self.ack_sender.build_out_frame(&mut frame, now);
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
		if self.retransmitter.disconnected(now) {
			Some(DisconnectedReason::ByRetryLimit)
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
