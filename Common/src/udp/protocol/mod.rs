use std::time::Instant;

use crate::udp::protocol::frame::{Frame, FrameId};
use std::fmt::Debug;

pub mod codec;
pub mod frame;
pub mod commands;
pub mod reliable;
pub mod others;
pub mod congestion;
pub mod relay;
pub mod disconnect;


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
pub trait FrameBuilder: Debug {
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
	fn disconnected(&mut self, now: &Instant) -> bool;
}



