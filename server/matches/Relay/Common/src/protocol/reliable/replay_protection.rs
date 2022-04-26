use crate::protocol::frame::input::InFrame;
pub use crate::protocol::frame::FrameId;
use crate::protocol::{MAX_FRAME_PER_SECONDS, NOT_EXIST_FRAME_ID};

///
/// Фильтрация уже принятых фреймов
///
/// - храним N идентификаторов фреймоа
/// - если пришел очень старый фрейм, которые уже не влазить в буфер - то мы не можем однозначно
/// сказать был ли он или нет, считаем что не было
///
#[derive(Debug)]
pub struct FrameReplayProtection {
	pub max_frame_id: FrameId,
	pub received_frames: [FrameId; FrameReplayProtection::BUFFER_SIZE],
}

impl Default for FrameReplayProtection {
	fn default() -> Self {
		Self {
			max_frame_id: 0,
			received_frames: [NOT_EXIST_FRAME_ID; FrameReplayProtection::BUFFER_SIZE],
		}
	}
}

impl FrameReplayProtection {
	pub const BUFFER_SIZE: usize = MAX_FRAME_PER_SECONDS * 20;

	///
	/// Отметить фрейм как принятый и проверить его статус
	///
	pub fn set_and_check(&mut self, frame: &InFrame) -> Result<bool, ()> {
		let frame_id = frame.get_original_frame_id();

		if frame_id > self.max_frame_id {
			self.max_frame_id = frame_id;
		}

		// нет возможности проверить статус
		if (frame_id + FrameReplayProtection::BUFFER_SIZE as u64) < self.max_frame_id {
			return Err(());
		}

		let index = frame_id as usize % FrameReplayProtection::BUFFER_SIZE;
		let stored_frame_id = self.received_frames[index];

		// такой фрейм уже был
		if stored_frame_id == frame_id {
			return Ok(true);
		}

		// если в ячейке буфера сохранен id более старого фрейма - то перезаписываем его
		// иначе - в ячейки уже более новый пакет и статус текущего пакета нельзя определить
		if frame_id > stored_frame_id {
			self.received_frames[index] = frame_id;
			Ok(false)
		} else {
			Err(())
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::frame::input::InFrame;
	use crate::protocol::reliable::replay_protection::FrameReplayProtection;

	#[test]
	fn should_protection_replay() {
		let mut protection = FrameReplayProtection::default();
		let frame_a = InFrame::new(1000);
		assert!(!protection.set_and_check(&frame_a).unwrap());
		assert!(protection.set_and_check(&frame_a).unwrap());
	}

	#[test]
	fn should_disconnect_when_very_old_frame() {
		let mut protection = FrameReplayProtection::default();
		let frame_a = InFrame::new(1000 + FrameReplayProtection::BUFFER_SIZE as u64);
		let frame_b = InFrame::new(10);
		assert!(!protection.set_and_check(&frame_a).unwrap());
		assert!(protection.set_and_check(&frame_b).is_err());
	}

	#[test]
	fn should_protection_replay_check_all() {
		let mut protection = FrameReplayProtection::default();
		for i in 1..(FrameReplayProtection::BUFFER_SIZE * 2) as u64 {
			let frame = InFrame::new(i);
			assert!(!protection.set_and_check(&frame).unwrap());
			assert!(protection.set_and_check(&frame).unwrap());
		}
	}

	#[test]
	fn should_protection_replay_check_prev_packets() {
		let mut protection = FrameReplayProtection::default();
		for i in 1..FrameReplayProtection::BUFFER_SIZE as u64 {
			let frame = InFrame::new(i);
			protection.set_and_check(&frame).unwrap();
			if i > 2 {
				for j in 1..i {
					let frame = InFrame::new(j);
					assert!(protection.set_and_check(&frame).unwrap());
				}
			}
		}
	}
}
