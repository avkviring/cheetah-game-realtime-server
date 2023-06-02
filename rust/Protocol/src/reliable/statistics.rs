use std::time::{Duration, Instant};

use generic_array::typenum::U8;
use prometheus::local::LocalIntCounter;

use crate::collections::event_collector_by_time::EventCollectorByTime;
use crate::frame::FrameId;

///
/// Сбор статистики по повторно отправленным пакетам
///
/// - количество повторно отправленных фреймов
/// - количество излишне повторно отправленных фреймов
///
#[derive(Debug)]
pub struct RetransmitStatistics {
	//
	// Статистика по излишне отправленным фреймам
	//
	redundant_events_collector: EventCollectorByTime<u8, U8>,

	///
	/// Статистика по повторно отправленным фреймам
	///
	retransmit_events_collector: EventCollectorByTime<u8, U8>,

	counter: LocalIntCounter, //already_processed_frames: LruCache<FrameId, bool>,
	                          //acked_original_frames: LruCache<FrameId, bool>,
}

impl RetransmitStatistics {
	//const FRAMES_STORAGE_LIMIT: usize = 2048;

	///
	/// Пометка ячейки в [`redundant_frames_measurements`] как не занятой
	///
	const EMPTY_MEASUREMENT_MARK: u8 = u8::MAX;

	///
	/// Время измерения для одной ячейки в [`redundant_frames_measurements`]
	///
	const MEASURE_DURATION: Duration = Duration::from_millis(5000);

	#[must_use]
	pub fn new(counter: LocalIntCounter) -> Self {
		Self {
			redundant_events_collector: EventCollectorByTime::new(0, 0, RetransmitStatistics::EMPTY_MEASUREMENT_MARK, RetransmitStatistics::MEASURE_DURATION),
			retransmit_events_collector: EventCollectorByTime::new(0, 0, RetransmitStatistics::EMPTY_MEASUREMENT_MARK, RetransmitStatistics::MEASURE_DURATION),
			counter,
			//already_processed_frames: LruCache::new(RetransmitStatistics::FRAMES_STORAGE_LIMIT),
			//acked_original_frames: LruCache::new(RetransmitStatistics::FRAMES_STORAGE_LIMIT),
		}
	}

	///
	/// Сбор статистики количества повторно подтверждаемых фреймов
	///
	/// - повторное подтверждение фрейма - это подтверждение исходного фрейма несколькими повторно отправленными фреймами
	///
	pub fn on_ack_received(&mut self, _frame_id: FrameId, _now: Instant) {
		// self.redundant_events_collector.switch_measure_position(now);

		// фрейм уже учтен в статистики - выходим
		// if self.already_processed_frames.contains(&frame_id) {
		// 	return;
		// }
		// self.already_processed_frames.put(frame_id, true);

		// если фрейм не подтвержден - подтверждаем и выходим
		// так как нам необходимо считать повторные подтверждения
		// if !self.acked_original_frames.contains(&frame_id) {
		// 	self.acked_original_frames.put(frame_id, true);
		// 	return;
		// }

		// self.redundant_events_collector.on_event(now);
	}

	pub fn on_retransmit_frame(&mut self, now: Instant) {
		self.counter.inc();
		self.retransmit_events_collector.on_event(now);
	}

	///
	/// Количество повторных излишних отправленных фреймов (скользящее среднее)
	///
	pub fn get_average_redundant_frames(&mut self, now: Instant) -> Option<usize> {
		self.redundant_events_collector.get_sum_and_count(now).map(|(sum, count)| (sum / count) as usize)
	}

	///
	/// Количество повторно отправленных фреймов
	///
	pub fn get_average_retransmit_frames(&mut self, now: Instant) -> Option<usize> {
		self.retransmit_events_collector.get_sum_and_count(now).map(|(sum, count)| (sum / count) as usize)
	}
}

#[cfg(test)]
mod tests {
	use prometheus::IntCounter;
	use std::ops::Add;
	use std::time::Instant;

	use crate::reliable::statistics::RetransmitStatistics;

	#[test]
	///
	/// Если не было достаточного числа измерений - нельзя получить среднее
	///
	fn redundant_should_return_none_in_redundant_statistics() {
		let mut statistics = RetransmitStatistics::new(IntCounter::new("name", "help").unwrap().local());
		let now = Instant::now();
		statistics.on_ack_received(1, now);
		assert!(matches!(statistics.get_average_redundant_frames(now), None));
	}

	///
	/// Подтверждение от одного и того же фрейма не должно учитываться несколько раз в статистике
	///
	#[test]
	fn redundant_should_return_average_1() {
		// let mut statistics = RetransmitStatistics::default();
		// let now = Instant::now();
		// statistics.on_ack_received(1, now);
		// statistics.on_ack_received(1, now);
		// statistics.on_ack_received(1, now);
		// let now = now.add(RetransmitStatistics::MEASURE_DURATION);
		// assert!(matches!(statistics.get_average_redundant_frames(now), Some(v) if v ==0));
	}

	///
	/// Однократное подтверждение двух фреймов, раз фреймы разные - то среднее равное нулю
	///
	#[test]
	fn redundant_should_return_average_2() {
		// let mut statistics = RetransmitStatistics::default();
		// let now = Instant::now();
		// statistics.on_ack_received(1, now);
		// statistics.on_ack_received(2, now);
		// let now = now.add(RetransmitStatistics::MEASURE_DURATION);
		// assert!(matches!(statistics.get_average_redundant_frames(now), Some(v) if v ==0));
	}

	///
	/// Подтверждение одного фрейма тремя фреймами, следующие два считаем повторным
	///
	// #[test]
	// fn redundant_should_return_average_3() {
	// 	let mut statistics = RetransmitStatistics::default();
	// 	let now = Instant::now();
	// 	statistics.on_ack_received(1, now);
	// 	statistics.on_ack_received(2, now);
	// 	statistics.on_ack_received(3, now);
	//
	// 	let now = now.add(RetransmitStatistics::MEASURE_DURATION);
	// 	assert!(matches!(statistics.get_average_redundant_frames(now), Some(v) if v ==2));
	// }

	///
	/// Проверяем работу несколько измерений
	///
	// #[test]
	// fn redundant_should_return_average_different_cells() {
	// 	let mut statistics = RetransmitStatistics::default();
	// 	let now = Instant::now();
	//
	// 	// 2 в первую ячейку
	// 	statistics.on_ack_received(1, now);
	// 	statistics.on_ack_received(2, now);
	// 	statistics.on_ack_received(3, now);
	// 	let now = now.add(RetransmitStatistics::MEASURE_DURATION);
	//
	// 	// 4 во вторую ячейку
	// 	statistics.on_ack_received(10, now);
	// 	statistics.on_ack_received(11, now);
	// 	statistics.on_ack_received(12, now);
	// 	statistics.on_ack_received(13, now);
	// 	statistics.on_ack_received(14, now);
	//
	// 	let now = now.add(RetransmitStatistics::MEASURE_DURATION);
	// 	assert!(matches!(statistics.get_average_redundant_frames(now), Some(v) if v ==3));
	// }

	///
	/// Проверяем измерение количества повторно отправленных фреймов
	///
	#[test]
	fn retransmit_should_average() {
		let mut statistics = RetransmitStatistics::new(IntCounter::new("name", "help").unwrap().local());
		let now = Instant::now();

		statistics.on_retransmit_frame(now);
		statistics.on_retransmit_frame(now);
		statistics.on_retransmit_frame(now);

		let now = now.add(RetransmitStatistics::MEASURE_DURATION);
		assert!(matches!(statistics.get_average_retransmit_frames(now), Some(v) if v ==3));
	}
}
