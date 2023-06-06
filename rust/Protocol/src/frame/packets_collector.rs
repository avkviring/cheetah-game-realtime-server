use fnv::FnvHashMap;

use crate::frame::segment::{Segment, SEGMENT_SIZE};

pub const PACKET_SIZE: usize = 32768;

///
/// Сборка сегментов в пакет
///
#[derive(Default, Debug)]
pub struct PacketsCollector {
	packets: FnvHashMap<u64, PacketCollector>,
	ready_packet: Vec<u8>,
}

impl PacketsCollector {
	pub(crate) fn on_segment(&mut self, segment: &Segment) -> Result<Option<&[u8]>, ()> {
		let packet_collector = match self.packets.get_mut(&segment.packet_id) {
			None => {
				self.packets.insert(segment.packet_id, Default::default());
				self.packets.get_mut(&segment.packet_id).unwrap()
			}
			Some(packet_collector) => packet_collector,
		};

		match packet_collector.on_data(segment) {
			Ok(ready) => {
				if ready {
					let packet_collector = self.packets.remove(&segment.packet_id).unwrap();
					packet_collector.to_packet(self.ready_packet.as_mut())?;
					Ok(Some(self.ready_packet.as_slice()))
				} else {
					Ok(None)
				}
			}
			Err(_) => {
				self.packets.remove(&segment.packet_id);
				Err(())
			}
		}
	}
}

#[derive(Default, Debug)]
struct PacketCollector {
	segments: FnvHashMap<u8, heapless::Vec<u8, SEGMENT_SIZE>>,
}

impl PacketCollector {
	pub(crate) fn on_data(&mut self, segment: &Segment) -> Result<bool, ()> {
		let vec = heapless::Vec::from_slice(&segment.body[0..segment.body_size])?;
		self.segments.insert(segment.current_segment, vec);
		Ok(self.segments.len() == segment.count_segments as usize)
	}

	pub(crate) fn to_packet(self, out: &mut Vec<u8>) -> Result<(), ()> {
		out.clear();
		for i in 0..self.segments.len() {
			match self.segments.get(&(i as u8)) {
				None => return Err(()),
				Some(data) => {
					out.extend_from_slice(data.as_slice());
				}
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::frame::packets_collector::PacketsCollector;
	use crate::frame::segment::Segment;

	#[test]
	fn should_dont_collect_packet() {
		let mut collector = PacketsCollector::default();
		assert!(collector.on_segment(&Segment::new(2, 2, 0, &[1, 2, 3])).unwrap().is_none());
	}

	#[test]
	fn should_collect_packet() {
		let mut collector = PacketsCollector::default();
		assert_eq!(collector.on_segment(&Segment::new(1, 1, 0, &[1, 2, 3])).unwrap().unwrap(), &[1, 2, 3]);
	}

	#[test]
	fn should_collect_packet_from_2_segment() {
		let mut collector = PacketsCollector::default();
		collector.on_segment(&Segment::new(2, 2, 0, &[1, 2, 3])).unwrap();
		assert_eq!(collector.on_segment(&Segment::new(2, 2, 1, &[4, 5, 6])).unwrap().unwrap(), &[1, 2, 3, 4, 5, 6]);
	}

	#[test]
	fn should_collect_packet_from_2_segment_with_reverse_order() {
		let mut collector = PacketsCollector::default();
		collector.on_segment(&Segment::new(2, 2, 1, &[4, 5, 6])).unwrap();
		assert_eq!(collector.on_segment(&Segment::new(2, 2, 0, &[1, 2, 3])).unwrap().unwrap(), &[1, 2, 3, 4, 5, 6])
	}
}
