use crate::frame::Frame;
pub use crate::frame::FrameId;
use crate::NOT_EXIST_FRAME_ID;

///
/// Фильтрация уже принятых фреймов
///
/// - храним N идентификаторов фрейма
/// - если пришел очень старый фрейм, которые уже не влазить в буфер - то мы не можем однозначно
/// сказать был ли он или нет, считаем что не было
///
#[derive(Debug)]
pub struct FrameReplayProtection {
	pub received_frames: Vec<FrameId>,
}

impl Default for FrameReplayProtection {
	fn default() -> Self {
		let mut vec = Vec::with_capacity(FrameReplayProtection::BUFFER_SIZE);
		vec.resize(FrameReplayProtection::BUFFER_SIZE, NOT_EXIST_FRAME_ID);
		Self { received_frames: vec }
	}
}

impl FrameReplayProtection {
	pub const BUFFER_SIZE: usize = 16384;

	///
	/// Отметить фрейм как принятый и проверить его статус
	///
	#[allow(clippy::result_unit_err)]
	#[allow(clippy::cast_possible_truncation)]
	pub fn set_and_check(&mut self, frame: &Frame) -> Result<bool, ()> {
		let frame_id = frame.get_original_frame_id();

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
	use crate::frame::Frame;
	use crate::reliable::replay_protection::FrameReplayProtection;

	#[test]
	fn should_protection_replay() {
		let mut protection = FrameReplayProtection::default();
		let frame_a = Frame::new(0, 1000, false, Default::default());
		assert!(!protection.set_and_check(&frame_a).unwrap());
		assert!(protection.set_and_check(&frame_a).unwrap());
	}

	#[test]
	fn should_disconnect_when_very_old_frame() {
		let mut protection = FrameReplayProtection::default();
		let frame_a = Frame::new(0, 1000 + FrameReplayProtection::BUFFER_SIZE as u64, false, Default::default());
		let frame_b = Frame::new(0, 1000, false, Default::default());
		assert!(!protection.set_and_check(&frame_a).unwrap());
		protection.set_and_check(&frame_b).unwrap_err();
	}

	#[test]
	fn should_protection_replay_check_all() {
		let mut protection = FrameReplayProtection::default();
		for i in 1..(FrameReplayProtection::BUFFER_SIZE * 2) as u64 {
			let frame = Frame::new(0, i, false, Default::default());
			assert!(!protection.set_and_check(&frame).unwrap());
			assert!(protection.set_and_check(&frame).unwrap());
		}
	}

	#[test]
	fn should_protection_replay_check_prev_packets() {
		let mut protection = FrameReplayProtection::default();
		for i in 1..FrameReplayProtection::BUFFER_SIZE as u64 {
			let frame = Frame::new(0, i, false, Default::default());
			protection.set_and_check(&frame).unwrap();
			if i > 2 {
				for j in 1..i {
					let frame = Frame::new(0, j, false, Default::default());
					assert!(protection.set_and_check(&frame).unwrap());
				}
			}
		}
	}
}
