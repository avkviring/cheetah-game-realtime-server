use snap::Error;

pub fn packet_compress(input: &[u8]) -> Result<Vec<u8>, ()> {
	let mut encoder = snap::raw::Encoder::new();
	encoder.compress_vec(input).map_err(|e| ())
}

pub fn packet_decompress(input: &[u8]) -> Result<Vec<u8>, ()> {
	let mut decoder = snap::raw::Decoder::new();
	decoder.decompress_vec(input).map_err(|e| ())
}