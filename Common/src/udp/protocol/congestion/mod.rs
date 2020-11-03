use std::ops::Sub;
use std::time::{Duration, Instant};

use crate::udp::protocol::others::rtt::RoundTripTime;
use crate::udp::protocol::reliable::retransmit::Retransmitter;

///
/// Контроль скорости для устранения перегрузок канала
///
#[derive(Default)]
pub struct CongestionControl {
	last_balanced: Option<Instant>,
	
}

impl CongestionControl {
	///
	/// Периоды между балансировками параметров
	///
	pub const REBALANCE_PERIOD: Duration = Duration::from_millis(500);
	
	
	pub fn rebalance(&mut self,
					 now: &Instant,
					 rtt: &dyn RoundTripTime,
					 retransmitter: &mut dyn Retransmitter,
	) {
		if !self.can_rebalance(now) {
			return;
		}
		
		self.rebalance_ack_timeout(now, retransmitter, rtt);
	}
	
	///
	/// Балансируем время ожидания ack для пакета
	///
	fn rebalance_ack_timeout(&mut self, now: &Instant, retransmitter: &mut dyn Retransmitter, rtt: &dyn RoundTripTime) {
		let average_rtt = rtt.get_rtt();
		if let Option::Some(average_rtt) = average_rtt {
			let koeff = match retransmitter.get_redundant_frames_percent(now) {
				None => { 1.5 }
				Some(percent) => match percent {
					0.0..=0.1 => 1.1,
					0.1..=0.2 => 1.5,
					0.2..=0.5 => 2.0,
					0.5..=0.8 => 2.5,
					_ => { 3.0 }
				},
			};
			let new_retransmit_timeout = average_rtt.mul_f64(koeff);
			
			retransmitter.set_ack_wait_duration(new_retransmit_timeout);
		}
	}
	
	
	fn can_rebalance(&mut self, now: &Instant) -> bool {
		let start_time = self.last_balanced.get_or_insert(now.clone());
		if now.sub(*start_time) >= CongestionControl::REBALANCE_PERIOD {
			self.last_balanced.replace(now.clone());
			true
		} else {
			false
		}
	}
}


#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};
	
	use mockall::predicate;
	
	use crate::udp::protocol::congestion::CongestionControl;
	use crate::udp::protocol::others::rtt::MockRoundTripTime;
	use crate::udp::protocol::reliable::retransmit::MockRetransmitter;
	
	#[test]
	pub fn should_invoke_set_ack_wait_duration() {
		let mut congestion = CongestionControl::default();
		let now = Instant::now();
		let mut retransmitter = setup_retransmitter(vec![0.15]);
		let rtt = setup_rtt(vec![Duration::from_millis(2)]);
		
		retransmitter.expect_set_ack_wait_duration()
			.times(1)
			.with(predicate::eq(Duration::from_millis(3)))
			.returning(|duration| ());
		
		
		congestion.rebalance(&now, rtt.as_ref(), retransmitter.as_mut());
		let now = now.add(CongestionControl::REBALANCE_PERIOD);
		congestion.rebalance(&now, rtt.as_ref(), retransmitter.as_mut());
		
		retransmitter.checkpoint();
	}
	
	
	fn setup_retransmitter(values: Vec<f64>) -> Box<MockRetransmitter> {
		let mut retransmitter = MockRetransmitter::new();
		values.into_iter().for_each(|v| {
			retransmitter
				.expect_get_redundant_frames_percent()
				.returning(move |_| Option::Some(v));
		});
		
		Box::new(retransmitter)
	}
	
	fn setup_rtt(vec: Vec<Duration>) -> Box<MockRoundTripTime> {
		let mut rtt = MockRoundTripTime::new();
		vec.into_iter().for_each(|v| {
			rtt
				.expect_get_rtt()
				.returning(move || Option::Some(v));
		});
		Box::new(rtt)
	}
}
