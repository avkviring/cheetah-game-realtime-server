pub fn packet_compress(input: &[u8]) -> Result<Vec<u8>, ()> {
	let mut encoder = snap::raw::Encoder::new();
	encoder.compress_vec(input).map_err(|_| ())
}

pub fn packet_decompress(input: &[u8]) -> Result<Vec<u8>, ()> {
	let mut decoder = snap::raw::Decoder::new();
	decoder.decompress_vec(input).map_err(|_| ())
}

#[cfg(test)]
mod tests {
	use easybench::bench;
	use crate::udp::protocol::codec::compress::{packet_compress, packet_decompress};
	
	#[test]
	fn bench_compress() {
		println!("{}", bench(|| {
			let original = vec![1, 2, 3, 4, 4, 3, 2, 4, 5, 6, 7, 5, 4, 3, 4, 5, 7, 7, 8, 5, 4, 2, 3, 4, 5, 6, 7, 8];
			let compressed = packet_compress(&original).unwrap();
			packet_decompress(&compressed).unwrap();
		}))
	}
	
	#[test]
	fn should_compress() {
		let original = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
		let compressed = packet_compress(&original).unwrap();
		let decompressed = packet_decompress(&compressed).unwrap();
		assert_eq!(original, decompressed)
	}
}