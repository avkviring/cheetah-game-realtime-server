use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::net::SocketAddr;
use std::ops::{Add, Div, Sub};
use std::time::{Duration, Instant};

use rand::Rng;

///
/// Эмуляция характеристик сети
///
#[derive(Debug, Default)]
pub struct NetworkLatencyEmulator {
	///
	/// Вероятность потери пакета
	///
	drop_probability: Option<f64>,
	///
	/// Действия режима потери пакета после наступления вероятности потери пакета
	///
	drop_time: Option<Duration>,
	///
	/// RTT (время прохождения пакета туда-обратно
	///
	rtt: Option<Duration>,
	///
	/// Процент случайности в RTT - 0..1
	///
	rtt_dispersion: Option<f64>,

	///
	/// Время начала потери пакетов (отказа сети)
	///
	drop_start: Option<Instant>,
	out_queue: BinaryHeap<BinaryFrame>,
	in_queue: BinaryHeap<BinaryFrame>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct BinaryFrame {
	time: Instant,
	addr: Option<SocketAddr>,
	buffer: Vec<u8>,
}

impl Ord for BinaryFrame {
	fn cmp(&self, other: &Self) -> Ordering {
		other.time.cmp(&self.time)
	}
}

impl PartialOrd for BinaryFrame {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		other.time.partial_cmp(&self.time)
	}
}

impl NetworkLatencyEmulator {
	///
	/// Получаем данные из сокета и решаем отдавать ли их или использовать очередь для эмуляции характеристик сети
	///
	pub fn schedule_in(&mut self, now: &Instant, buffer: &[u8]) {
		if !self.is_drop_time(now) {
			let time = self.get_schedule_time(now);
			self.in_queue.push(BinaryFrame {
				time,
				addr: None,
				buffer: buffer.to_vec(),
			});
		}
	}

	///
	/// Получить данные для клиента с учетом всех параметров эмуляции
	///
	pub fn get_in(&mut self, now: &Instant) -> Option<Vec<u8>> {
		match self.in_queue.peek() {
			None => None,
			Some(frame) => {
				if *now >= frame.time {
					let frame = self.in_queue.pop().unwrap();
					Some(frame.buffer)
				} else {
					None
				}
			}
		}
	}

	///
	/// Сохраняем данные для отправки, реальная отправка происходит с учетом всех характеристик эмулируемой сети
	///
	pub fn schedule_out(&mut self, now: &Instant, buffer: &[u8], addr: SocketAddr) {
		if !self.is_drop_time(now) {
			let time = self.get_schedule_time(now);
			self.out_queue.push(BinaryFrame {
				time,
				addr: Some(addr),
				buffer: buffer.to_vec(),
			});
		}
	}

	///
	/// Получаем данные для отправки в реальный сокет
	///
	pub fn get_out(&mut self, now: &Instant) -> Option<(Vec<u8>, SocketAddr)> {
		match self.out_queue.peek() {
			None => Option::None,
			Some(data) => {
				if *now >= data.time {
					let data = self.out_queue.pop().unwrap();
					Option::Some((data.buffer.clone(), data.addr.unwrap()))
				} else {
					Option::None
				}
			}
		}
	}

	fn get_schedule_time(&mut self, now: &Instant) -> Instant {
		let rtt = *self.rtt.as_ref().unwrap_or(&Duration::from_millis(0));
		let half_rtt = rtt.div(2);
		let mut time = now.add(half_rtt);
		let rand: f64 = rand::thread_rng().gen();
		let delta = half_rtt.mul_f64(self.rtt_dispersion.unwrap_or(0.0) * rand);
		if rand::thread_rng().gen() {
			time = time.add(delta);
		} else {
			time = time.sub(delta);
		}
		time
	}

	fn is_drop_time(&mut self, now: &Instant) -> bool {
		if let Some(drop_time) = &self.drop_start {
			if drop_time.add(self.drop_time.unwrap_or(Duration::from_millis(0))) > *now {
				return true;
			}
			self.drop_start = None;
		};

		match self.drop_probability {
			None => false,
			Some(drop_probability) => {
				if drop_probability > rand::thread_rng().gen() {
					self.drop_start = Option::Some(*now);
					true
				} else {
					false
				}
			}
		}
	}

	pub fn configure_rtt(&mut self, rtt: Duration, rtt_dispersion: f64) {
		self.rtt = Some(rtt);
		self.rtt_dispersion = Some(rtt_dispersion);
	}

	pub fn configure_drop(&mut self, drop_probability: f64, drop_time: Duration) {
		self.drop_probability = Some(drop_probability);
		self.drop_time = Some(drop_time)
	}
}

#[cfg(test)]
mod tests {
	use std::net::SocketAddr;
	use std::ops::{Add, Div, Sub};
	use std::str::FromStr;
	use std::time::{Duration, Instant};

	use crate::network::emulator::NetworkLatencyEmulator;

	///
	/// Если не заданы ограничения - все должно работать
	///
	#[test]
	fn should_work_with_default_config() {
		let mut emulator = NetworkLatencyEmulator::default();
		let in_buffer = vec![1, 2, 3, 4, 5];
		let out_buffer = vec![10, 11, 12];
		emulator.schedule_in(&Instant::now(), &in_buffer.as_slice());
		emulator.schedule_out(&Instant::now(), &out_buffer.as_slice(), SocketAddr::from_str("127.0.0.1:5050").unwrap());

		assert!(matches!(emulator.get_in(&Instant::now()), Some(buffer) if buffer==in_buffer));
		assert!(matches!(emulator.get_out(&Instant::now()), Some((buffer,_)) if buffer==out_buffer));
	}

	///
	/// Проверяем работу канала в режиме эмуляции rtt
	///
	#[test]
	fn should_rtt_emulation_for_out() {
		let mut emulator = NetworkLatencyEmulator::default();
		let mut now = Instant::now();
		let send_data = vec![1, 2, 3];
		let rtt = Duration::from_millis(1000);
		emulator.rtt = Some(rtt);
		emulator.schedule_out(&now, send_data.as_slice(), SocketAddr::from_str("127.0.0.1:5050").unwrap());

		assert!(matches!(emulator.get_out(&now), Option::None));

		// время задержки прошло - данные доступны для отправки
		now = now.add(rtt.div(2));
		assert!(matches!(emulator.get_out(&now), Option::Some(_)));

		// больше данных нет
		assert!(matches!(emulator.get_out(&now), Option::None));
	}

	#[test]
	fn should_rtt_emulation_for_in() {
		let mut emulator = NetworkLatencyEmulator::default();
		let mut now = Instant::now();
		let send_data = vec![1, 2, 3];
		let rtt = Duration::from_millis(1000);
		emulator.rtt = Some(rtt);
		emulator.schedule_in(&now, send_data.as_slice());
		assert!(matches!(emulator.get_in(&now), Option::None));

		// время задержки прошло - данные доступны для отправки
		now = now.add(rtt.div(2));
		assert!(matches!(emulator.get_in(&now), Option::Some(_)));

		// больше данных нет
		assert!(matches!(emulator.get_in(&now), Option::None));
	}

	#[test]
	fn should_calculate_rtt_with_dispersion() {
		let mut emulator = NetworkLatencyEmulator::default();
		let rtt = Duration::from_millis(100);
		let half_rtt = rtt.div_f64(2.0);
		let rtt_dispersion = 0.1;
		emulator.rtt = Some(rtt);
		emulator.rtt_dispersion = Some(rtt_dispersion);
		let now = Instant::now();
		let mut count_not_equal_half = 0;
		for _ in 0..1000 {
			let instant = emulator.get_schedule_time(&now);
			let delta = instant.sub(now);
			assert!(delta.as_secs_f64() >= half_rtt.as_secs_f64() * (1.0 - rtt_dispersion));
			assert!(delta.as_secs_f64() <= half_rtt.as_secs_f64() * (1.0 + rtt_dispersion));
			if delta != half_rtt {
				count_not_equal_half += 1;
			}
		}

		assert!(count_not_equal_half > 0)
	}

	#[test]
	fn should_drop_packet() {
		let mut emulator = NetworkLatencyEmulator::default();
		emulator.drop_probability = Option::Some(0.5);
		emulator.drop_time = Option::Some(Duration::from_millis(10));
		let buffer = vec![1, 2, 3];

		let count = 1000;
		let mut in_dropped_count = 0;
		let mut out_dropped_count = 0;
		let mut now = Instant::now();
		for _ in 0..count {
			now = now.add(Duration::from_millis(1));
			emulator.schedule_in(&now, &buffer.as_slice());

			if let None = emulator.get_in(&now) {
				in_dropped_count += 1;
			}

			emulator.schedule_out(&now, &buffer.as_slice(), SocketAddr::from_str("127.0.0.1:5050").unwrap());
			if let None = emulator.get_out(&now) {
				out_dropped_count += 1;
			}
		}
		assert!(in_dropped_count > 0);
		assert!(in_dropped_count < count);

		assert!(out_dropped_count > 0);
		assert!(out_dropped_count < count);
	}

	#[test]
	fn should_drop_packet_with_time() {
		let mut emulator = NetworkLatencyEmulator::default();
		emulator.drop_probability = Option::Some(1.0);
		let drop_time = Duration::from_millis(10);
		emulator.drop_time = Option::Some(drop_time);

		let now = Instant::now();
		emulator.is_drop_time(&now);
		// проверяем установки времени эмуляции
		assert!(matches!(emulator.drop_start, Some(time) if time == now));

		// мы в зоне отказа сети, даже если вероятность отказа 0
		emulator.drop_probability = Option::Some(0.0);
		assert!(emulator.is_drop_time(&now));
		// выходим из зоны отказа сети
		assert!(!emulator.is_drop_time(&now.add(drop_time)));
	}

	#[test]
	fn should_keep_frame_order() {
		let mut emulator = NetworkLatencyEmulator::default();
		let rtt = Duration::from_millis(100);
		let half_rtt = rtt.div(2);
		emulator.configure_rtt(rtt, 0.0);

		let frame_1 = vec![1];
		let frame_2 = vec![2];
		let now = Instant::now();

		emulator.schedule_in(&now, &frame_1.as_slice());
		emulator.schedule_in(&now, &frame_2.as_slice());
		emulator.schedule_out(&now, &frame_1.as_slice(), SocketAddr::from_str("127.0.0.1:5050").unwrap());
		emulator.schedule_out(&now, &frame_2.as_slice(), SocketAddr::from_str("127.0.0.1:5050").unwrap());

		let now = now.add(half_rtt.sub(Duration::from_millis(1)));
		assert!(matches!(emulator.get_in(&now), None));
		assert!(matches!(emulator.get_out(&now), None));

		let now = now.add(Duration::from_millis(1));
		assert!(matches!(emulator.get_in(&now),Some(frame) if frame==frame_1 ));
		assert!(matches!(emulator.get_in(&now),Some(frame) if frame==frame_2 ));

		assert!(matches!(emulator.get_out(&now),Some((frame,_)) if frame==frame_1));
		assert!(matches!(emulator.get_out(&now),Some((frame,_)) if frame==frame_2));
	}
}
