use std::ops::{AddAssign, Sub};
use std::time::{Duration, Instant};

use generic_array::sequence::GenericSequence;
use generic_array::{ArrayLength, GenericArray};
use std::fmt::Debug;

///
///  Сохраняет количество событий по времени
/// - используется кольцевой буфер - сохраняется N измерений
/// - каждое измерение собирается определенное время, после этого сумма событий записывается в буфер и начинается следующее измерение
///
///
#[derive(Debug)]
pub struct EventCollectorByTime<T: Debug, N: ArrayLength> {
	current_value: T,
	default_value: T,
	unset_cell_value: T,

	start_measurement_time: Option<Instant>,

	///
	/// Хранение агрегированных значений
	///
	ring_buffer: GenericArray<T, N>,

	///
	/// Время агрегации значений
	///
	measure_time: Duration,

	///
	/// Позиция предыдущего сохранения агрегированного значения
	///
	position: usize,
}

impl<T: Copy + PartialEq + AddAssign<T> + Int + Debug, N: ArrayLength> EventCollectorByTime<T, N> {
	pub fn new(current_value: T, default_value: T, unset_cell_value: T, aggregation_time: Duration) -> Self {
		Self {
			default_value,
			current_value,
			unset_cell_value,
			ring_buffer: GenericArray::generate(|_| unset_cell_value),
			start_measurement_time: Default::default(),
			measure_time: aggregation_time,
			position: Default::default(),
		}
	}

	pub fn switch_measure_position(&mut self, now: Instant) {
		let start_time = match self.start_measurement_time {
			None => {
				self.start_measurement_time = Some(now);
				now
			}
			Some(time) => time,
		};

		if now.sub(start_time) >= self.measure_time {
			self.start_measurement_time = Some(now);
			let mut new_position = self.position + 1;
			if new_position == self.ring_buffer.len() {
				new_position = 0;
			}
			self.ring_buffer[self.position] = self.current_value;
			self.position = new_position;
			self.current_value = self.default_value;
		}
	}

	pub fn get_sum_and_count(&mut self, now: Instant) -> Option<(T, u8)> {
		self.switch_measure_position(now);
		let mut sum = self.default_value;
		let mut count = 0;
		for i in 0..self.ring_buffer.len() {
			let value = self.ring_buffer[i];
			if value != self.unset_cell_value {
				sum += value;
				count += 1;
			}
		}
		if count == 0 {
			None
		} else {
			Some((sum, count))
		}
	}

	pub fn on_event(&mut self, now: Instant) {
		self.switch_measure_position(now);
		let option = self.current_value.add_with_overflow_control(T::one());
		if let Some(value) = option {
			self.current_value = value;
		}
	}
}

pub trait Int: Sized {
	fn one() -> Self;
	fn add_with_overflow_control(&self, b: Self) -> Option<Self>;
}

impl Int for u32 {
	fn one() -> Self {
		1
	}

	fn add_with_overflow_control(&self, b: Self) -> Option<Self> {
		self.checked_add(b)
	}
}

impl Int for u16 {
	fn one() -> Self {
		1
	}
	fn add_with_overflow_control(&self, b: Self) -> Option<Self> {
		self.checked_add(b)
	}
}

impl Int for u8 {
	fn one() -> Self {
		1
	}
	fn add_with_overflow_control(&self, b: Self) -> Option<Self> {
		self.checked_add(b)
	}
}

#[cfg(test)]
mod tests {
	use std::ops::{Add, AddAssign};
	use std::time::{Duration, Instant};

	use generic_array::typenum::U8;

	use crate::collections::event_collector_by_time::EventCollectorByTime;

	#[test]
	///
	/// Если не прошло время агрегации - сумма и количество не может быть определено
	///
	pub(crate) fn should_return_none_if_not_enough_duration() {
		let (mut collector, _) = setup();
		let now = Instant::now();
		collector.on_event(now);
		assert!(matches!(collector.get_sum_and_count(now), None));
	}

	///
	/// Проверяем агрегирование в рамках одного временного диапазона
	///
	#[test]
	pub(crate) fn should_return_sum_and_count_for_one_aggregation() {
		let (mut collector, duration) = setup();
		let now = Instant::now();
		collector.on_event(now);
		collector.on_event(now);
		collector.on_event(now);
		assert!(matches!(collector.get_sum_and_count(now.add(duration)), Some((sum, count)) if sum == 3 && count == 1));
	}

	///
	/// Проверяем агрегирование в рамках двух временных диапазонов
	///
	#[test]
	pub(crate) fn should_return_sum_and_count_for_two_aggregation() {
		let (mut collector, duration) = setup();
		let mut now = Instant::now();
		collector.on_event(now);
		collector.on_event(now);
		collector.on_event(now);
		now.add_assign(duration);
		collector.on_event(now);
		collector.on_event(now);
		assert!(matches!(collector.get_sum_and_count(now.add(duration)), Some((sum, count)) if sum == 5 && count == 2));
	}

	fn setup() -> (EventCollectorByTime<u8, U8>, Duration) {
		let duration = Duration::from_millis(1);
		let collector = EventCollectorByTime::<u8, U8>::new(0, 0, u8::MAX, duration);
		(collector, duration)
	}
}
