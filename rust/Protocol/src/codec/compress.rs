use snap::Error;

pub fn packet_compress(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
	let mut encoder = snap::raw::Encoder::new();
	encoder.compress(input, output)
}

pub fn packet_decompress(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
	let mut decoder = snap::raw::Decoder::new();
	decoder.decompress(input, output)
}

#[cfg(test)]
mod tests {
	use crate::codec::compress::{packet_compress, packet_decompress};

	#[test]
	fn should_compress() {
		let original = vec![1, 2, 3, 4, 4, 3, 2, 4, 5, 6, 7, 5, 4, 3, 4, 5, 7, 7, 8, 5, 4, 2, 3, 4, 5, 6, 7, 8];
		let mut compressed = [0; 100];
		let compressed_size = packet_compress(&original, &mut compressed).unwrap();

		let mut decompressed = [0; 100];
		let decompressed_size = packet_decompress(&compressed[0..compressed_size], &mut decompressed).unwrap();
		assert_eq!(original.as_slice(), &decompressed[0..decompressed_size]);
	}
}
