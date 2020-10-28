use std::cmp::{max, min};
use std::collections::VecDeque;
use std::ops::Sub;
use std::time::{Duration, Instant};

use crate::udp::protocol::others::rtt::RoundTripTimeHandler;
use crate::udp::protocol::reliable::retransmit::Retransmitter;

///
/// Контроль скорости для устранения перегрузок канала
///
#[derive(Default)]
pub struct CongestionControl {
	pub last_balanced: Option<Instant>,
}

impl CongestionControl {
	///
	/// Периоды между балансировками параметров
	///
	pub const REBALANCE_PERIOD: Duration = Duration::from_millis(500);
	
	///
	/// Минимальное время ожидания ask
	///
	pub const MIN_ASK_TIMEOUT: Duration = Duration::from_millis(50);
	
	///
	/// Максимальное время ожидания ask
	///
	pub const MAX_ASK_TIMEOUT: Duration = Duration::from_millis(600);
	
	///
	/// Коррекция времени ASK ответа для учета непредвиденных ситуаций
	///
	pub const ASK_TIME_CORRECTION: Duration = Duration::from_millis(15);
	
	
	pub fn rebalance(&mut self, now: &Instant, rtt: &RoundTripTimeHandler, retransmitter: &mut Retransmitter) {
		if !self.is_time_to_rebalance(now) {
			return;
		}
		
		self.rebalance_ask_timeout(retransmitter);
	}
	
	///
	/// Балансируем время ожидания ask для пакета
	///
	fn rebalance_ask_timeout(&mut self, retransmitter: &mut Retransmitter) {
		// let average_rtt = self.calculate_average_rtt();
		// if let Option::Some(average_rtt) = average_rtt {
		// 	let new_retransmit_timeout = average_rtt + CongestionControl::ASK_TIME_CORRECTION.as_millis() as u64;
		// 	let new_retransmit_timeout = max(new_retransmit_timeout, CongestionControl::MIN_ASK_TIMEOUT.as_millis() as u64);
		// 	let new_retransmit_timeout = min(new_retransmit_timeout, CongestionControl::MAX_ASK_TIMEOUT.as_millis() as u64);
		// 	retransmitter.timeout = Duration::from_millis(new_retransmit_timeout);
		// }
	}
	
	
	pub fn is_time_to_rebalance(&mut self, now: &Instant) -> bool {
		match self.last_balanced {
			None => {
				self.last_balanced = Option::Some(now.clone());
				true
			}
			Some(ref last_balanced) if now.sub(last_balanced.clone()) >= CongestionControl::REBALANCE_PERIOD => {
				self.last_balanced = Option::Some(now.clone());
				true
			}
			_ => {
				false
			}
		}
	}
}


#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};
	
	use crate::udp::protocol::congestion::CongestionControl;
	use crate::udp::protocol::others::rtt::RoundTripTimeHandler;
	use crate::udp::protocol::reliable::retransmit::Retransmitter;
	
	
	
	
}
