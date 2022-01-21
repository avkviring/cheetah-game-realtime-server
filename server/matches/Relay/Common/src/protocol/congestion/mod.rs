use std::ops::Sub;
use std::time::{Duration, Instant};

use crate::protocol::others::rtt::RoundTripTime;
use crate::protocol::reliable::retransmit::Retransmitter;

///
/// Контроль скорости для устранения перегрузок канала
///
#[derive(Default, Debug)]
pub struct CongestionControl {
	last_balanced: Option<Instant>,
}

impl CongestionControl {
	///
	/// Периоды между балансировками параметров
	///
	pub const REBALANCE_PERIOD: Duration = Duration::from_millis(500);

	pub fn rebalance(&mut self, now: &Instant, rtt: &RoundTripTime, retransmitter: &mut Retransmitter) {
		if !self.can_rebalance(now) {
			return;
		}

		self.rebalance_ack_timeout(now, retransmitter, rtt);
	}

	///
	/// Балансируем время ожидания ack для пакета
	///
	fn rebalance_ack_timeout(&mut self, _now: &Instant, _retransmitter: &mut Retransmitter, _rtt: &RoundTripTime) {
		// let average_rtt = rtt.get_rtt();
		// if let Option::Some(average_rtt) = average_rtt {
		// 	let koeff = match retransmitter.get_redundant_frames_percent(now) {
		// 		None => 1.5,
		// 		Some(percent) => match (percent * 100.0) as u64 {
		// 			0..=10 => 1.1,
		// 			11..=20 => 1.5,
		// 			21..=30 => 2.0,
		// 			31..=80 => 2.5,
		// 			_ => 3.0,
		// 		},
		// 	};
		// 	let new_retransmit_timeout = average_rtt.mul_f64(koeff);
		// 		retransmitter.set_ack_wait_duration(new_retransmit_timeout);
		// }
	}

	fn can_rebalance(&mut self, now: &Instant) -> bool {
		let start_time = self.last_balanced.get_or_insert(*now);
		if now.sub(*start_time) >= CongestionControl::REBALANCE_PERIOD {
			self.last_balanced.replace(*now);
			true
		} else {
			false
		}
	}
}

#[cfg(test)]
mod tests {

	//#[test]
	// pub fn should_invoke_set_ack_wait_duration() {
	// 	let mut congestion = CongestionControl::default();
	// 	let now = Instant::now();
	// 	let mut retransmitter = setup_retransmitter(vec![0.15]);
	// 	let rtt = setup_rtt(vec![Duration::from_millis(2)]);
	//
	// 	retransmitter
	// 		.expect_set_ack_wait_duration()
	// 		.times(1)
	// 		.with(predicate::eq(Duration::from_millis(3)))
	// 		.returning(|_| ());
	//
	// 	congestion.rebalance(&now, rtt.as_ref(), retransmitter.as_mut());
	// 	let now = now.add(CongestionControl::REBALANCE_PERIOD);
	// 	congestion.rebalance(&now, rtt.as_ref(), retransmitter.as_mut());
	//
	// 	retransmitter.checkpoint();
	// }

	// fn setup_retransmitter(values: Vec<f64>) -> Box<MockRetransmitter> {
	// 	let mut retransmitter = MockRetransmitter::new();
	// 	values.into_iter().for_each(|v| {
	// 		retransmitter.expect_get_redundant_frames_percent().returning(move |_| Option::Some(v));
	// 	});
	//
	// 	Box::new(retransmitter)
	// }
	//
	// fn setup_rtt(vec: Vec<Duration>) -> Box<MockRoundTripTime> {
	// 	let mut rtt = MockRoundTripTime::new();
	// 	vec.into_iter().for_each(|v| {
	// 		rtt.expect_get_rtt().returning(move || Option::Some(v));
	// 	});
	// 	Box::new(rtt)
	// }
}
