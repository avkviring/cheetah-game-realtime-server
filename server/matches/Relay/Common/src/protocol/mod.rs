use std::fmt::Debug;
use std::time::Instant;

use crate::protocol::commands::input::InCommandsCollector;
use crate::protocol::commands::output::OutCommandsCollector;
use crate::protocol::congestion::CongestionControl;
use crate::protocol::disconnect::handler::DisconnectCommandHandler;
use crate::protocol::disconnect::watcher::DisconnectWatcher;
use crate::protocol::frame::{Frame, FrameId};
use crate::protocol::others::keep_alive::KeepAlive;
use crate::protocol::others::rtt::RoundTripTimeImpl;
use crate::protocol::reliable::ack::AckSender;
use crate::protocol::reliable::replay_protection::FrameReplayProtection;
use crate::protocol::reliable::retransmit::RetransmitterImpl;

pub mod codec;
pub mod commands;
pub mod congestion;
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
/// Обработчики входящих фреймов
///
pub trait FrameReceivedListener {
	fn on_frame_received(&mut self, frame: &Frame, now: &Instant);
}

///
/// Наполнение данными исходящего фрейма
///
pub trait FrameBuilder {
	///
	/// Есть ли собственные данные для отправки?
	///
	fn contains_self_data(&self, now: &Instant) -> bool;

	///
	/// Заполнить данными фрейм для отправки
	///
	fn build_frame(&mut self, frame: &mut Frame, now: &Instant);
}

///
/// Фрейм построен и готов для отправки
///
pub trait FrameBuiltListener {
	fn on_frame_built(&mut self, frame: &Frame, now: &Instant);
}

///
/// Статус разрыва связи в канале
///
pub trait DisconnectedStatus {
	fn disconnected(&self, now: &Instant) -> bool;
}

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
	pub retransmitter: RetransmitterImpl,
	pub disconnect_watcher: DisconnectWatcher,
	pub disconnect_handler: DisconnectCommandHandler,
	pub in_commands_collector: InCommandsCollector,
	pub out_commands_collector: OutCommandsCollector,
	pub rtt: RoundTripTimeImpl,
	pub keep_alive: KeepAlive,
	pub congestion_control: CongestionControl,
	pub in_frame_counter: u64,
}

impl Protocol {
	pub fn new(now: &Instant) -> Self {
		Self {
			next_frame_id: 1,
			disconnect_watcher: DisconnectWatcher::new(now),
			replay_protection: Default::default(),
			ack_sender: Default::default(),
			in_commands_collector: Default::default(),
			out_commands_collector: Default::default(),
			retransmitter: Default::default(),
			disconnect_handler: Default::default(),
			rtt: RoundTripTimeImpl::new(now),
			keep_alive: Default::default(),
			congestion_control: Default::default(),
			in_frame_counter: Default::default(),
		}
	}

	///
	/// Данный метод необходимо периодически вызывать
	/// для обработки внутренних данных
	///
	pub fn cycle(&mut self, now: &Instant) {
		self.congestion_control.rebalance(now, &self.rtt, &mut self.retransmitter);
	}

	///
	/// Обработка входящего фрейма
	///
	pub fn on_frame_received(&mut self, frame: Frame, now: &Instant) {
		self.in_frame_counter += 1;
		self.disconnect_watcher.on_frame_received(&frame, now);
		self.retransmitter.on_frame_received(&frame, now);
		if let Ok(replayed) = self.replay_protection.set_and_check(&frame, now) {
			if !replayed {
				self.disconnect_handler.on_frame_received(&frame, now);
				self.ack_sender.on_frame_received(&frame, now);
				self.rtt.on_frame_received(&frame, now);
				self.in_commands_collector.collect(frame);
			}
		}
	}

	///
	/// Создание фрейма для отправки
	///
	pub fn build_next_frame(&mut self, now: &Instant) -> Option<Frame> {
		match self.get_next_retransmit_frame(now) {
			None => {}
			Some(frame) => {
				return Option::Some(frame);
			}
		}

		let contains_data = self.ack_sender.contains_self_data(now)
			|| self.out_commands_collector.contains_self_data(now)
			|| self.disconnect_handler.contains_self_data(now)
			|| self.rtt.contains_self_data(now)
			|| self.keep_alive.contains_self_data(now);

		if contains_data {
			let mut frame = Frame::new(self.next_frame_id);
			self.next_frame_id += 1;

			self.ack_sender.build_frame(&mut frame, now);
			self.out_commands_collector.build_frame(&mut frame, now);
			self.disconnect_handler.build_frame(&mut frame, now);
			self.rtt.build_frame(&mut frame, now);
			self.keep_alive.build_frame(&mut frame, now);
			self.retransmitter.on_frame_built(&frame, now);
			Option::Some(frame)
		} else {
			Option::None
		}
	}

	///
	/// Разорвана ли связь?
	///
	pub fn disconnected(&self, now: &Instant) -> bool {
		self.retransmitter.disconnected(now)
			|| self.disconnect_watcher.disconnected(now)
			|| self.disconnect_handler.disconnected(now)
	}

	///
	/// Установлено ли соединения?
	///
	pub fn connected(&self, now: &Instant) -> bool {
		self.in_frame_counter > 0 && !self.disconnected(now)
	}

	pub fn get_next_retransmit_frame(&mut self, now: &Instant) -> Option<Frame> {
		let next_frame_id = self.next_frame_id + 1;
		match self.retransmitter.get_retransmit_frame(now, next_frame_id) {
			None => Option::None,
			Some(frame) => {
				self.next_frame_id = next_frame_id;
				Option::Some(frame)
			}
		}
	}
}
