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
	pub rtt: VecDeque<Duration>,
}

impl CongestionControl {
	///
	/// Периоды между балансировками параметров
	///
	pub const REBALANCE_PERIOD: Duration = Duration::from_millis(500);
	
	///
	/// Период для расчета среднего RTT
	///
	pub const RTT_AVG_PERIOD: Duration = Duration::from_millis(5000);
	
	///
	/// Минимальное количество RTT для которых производиться расчет среднего
	///
	pub const AVERAGE_RTT_MIN_LEN: usize = 5;
	
	///
	/// Минимальное время ожидания ask
	///
	pub const MIN_ASK_TIMEOUT: Duration = Duration::from_millis(50);
	
	///
	/// ASK по умолчанию
	///
	pub const DEFAULT_ASK_TIMEOUT: Duration = Duration::from_millis(300);
	
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
		
		self.collect_rtt(rtt);
		self.rebalance_ask_timeout(retransmitter);
	}
	
	///
	/// Балансируем время ожидания ask для пакета
	///
	fn rebalance_ask_timeout(&mut self, retransmitter: &mut Retransmitter) {
		let average_rtt = self.calculate_average_rtt();
		if let Option::Some(average_rtt) = average_rtt {
			let new_retransmit_timeout = average_rtt + CongestionControl::ASK_TIME_CORRECTION.as_millis() as u64;
			let new_retransmit_timeout = max(new_retransmit_timeout, CongestionControl::MIN_ASK_TIMEOUT.as_millis() as u64);
			let new_retransmit_timeout = min(new_retransmit_timeout, CongestionControl::MAX_ASK_TIMEOUT.as_millis() as u64);
			retransmitter.timeout = Duration::from_millis(new_retransmit_timeout);
		}
	}
	
	
	///
	/// Скользящий средний для rtt
	///
	fn calculate_average_rtt(&self) -> Option<u64> {
		if self.rtt.len() < CongestionControl::AVERAGE_RTT_MIN_LEN {
			Option::None
		} else {
			let sum_rtt: u64 = self.rtt.iter().map(|i| i.as_millis() as u64).sum();
			let average_rtt = sum_rtt / self.rtt.len() as u64;
			Option::Some(average_rtt)
		}
	}
	
	///
	/// Сохраняем текущее rtt для подсчета скользящего среднего
	///
	fn collect_rtt(&mut self, rtt: &RoundTripTimeHandler) {
		match rtt.rtt {
			None => {}
			Some(ref rtt) => {
				self.rtt.push_back(rtt.clone());
				if (self.rtt.len() as u128) * CongestionControl::REBALANCE_PERIOD.as_millis() >= CongestionControl::RTT_AVG_PERIOD.as_millis() {
					self.rtt.pop_front();
				}
			}
		}
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
	
	#[test]
	///
	/// Проверяем расчет среднего rtt
	///
	/// - учитываем время балансировки (не каждый вызов rebalance приводит к ребалансировки)
	/// - учитываем что для расчета среднего rtt количество измерение должно быть больше определенного значения
	///
	pub fn should_calculate_rtt_average() {
		let mut handler = CongestionControl::default();
		let mut now = Instant::now();
		let mut rtt = RoundTripTimeHandler::default();
		let mut retransmitter = Retransmitter::default();
		for i in 0..CongestionControl::AVERAGE_RTT_MIN_LEN {
			rtt.rtt = Option::Some(Duration::from_millis(i as u64));
			handler.rebalance(&now, &rtt, &mut retransmitter);
			now = now.add(CongestionControl::REBALANCE_PERIOD);
		}
		
		let average = handler.calculate_average_rtt();
		assert!(matches!(average, Some(average) if average==2));
	}
	
	///
	/// Проверяем соблюдение [CongestionControl::REBALANCE_PERIOD]
	/// - расчет среднего rtt не должен быть произведен если нет достаточного количество измерений
	/// - а их не должно быть, так как мы не соблюдаем период ребалансировки
	///
	#[test]
	pub fn should_not_calculate_rtt_average_when_no_period() {
		let mut handler = CongestionControl::default();
		let now = Instant::now();
		let mut rtt = RoundTripTimeHandler::default();
		let mut retransmitter = Retransmitter::default();
		for i in 0..CongestionControl::AVERAGE_RTT_MIN_LEN {
			rtt.rtt = Option::Some(Duration::from_millis(i as u64));
			handler.rebalance(&now, &rtt, &mut retransmitter);
		}
		
		let average = handler.calculate_average_rtt();
		assert!(matches!(average, Option::None));
	}
	
	
	#[test]
	///
	/// Проверяем балансировку ask_timeout
	/// - само значение не проверяем, так как алгоритм может измениться
	/// - проверяем только краевые условия
	///
	pub fn should_change_ask_timeout() {
		let mut handler = CongestionControl::default();
		let mut now = Instant::now();
		let mut rtt = RoundTripTimeHandler::default();
		
		let mut retransmitter = Retransmitter::default();
		retransmitter.timeout = Duration::from_millis(0);
		
		for i in 0..CongestionControl::AVERAGE_RTT_MIN_LEN {
			rtt.rtt = Option::Some(Duration::from_millis(i as u64));
			handler.rebalance(&now, &rtt, &mut retransmitter);
			now = now.add(CongestionControl::REBALANCE_PERIOD);
		}
		
		assert!(retransmitter.timeout >= CongestionControl::MIN_ASK_TIMEOUT);
		assert!(retransmitter.timeout <= CongestionControl::MAX_ASK_TIMEOUT);
	}
}
