use easybench::bench;
use cheetah_relay_common::udp::frame::codec::compress::{packet_compress, packet_decompress};

#[test]
fn bench_compress() {
	println!("{}", bench(|| {
		let original = vec![1, 2, 3, 4, 4, 3, 2, 4, 5, 6, 7, 5, 4, 3, 4, 5, 7, 7, 8, 5, 4, 2, 3, 4, 5, 6, 7, 8];
		let compressed = packet_compress(&original).unwrap();
		let decompressed = packet_decompress(&compressed).unwrap();
	}))
}

#[test]
fn should_compress() {
	let original = vec![0,0,0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1];
	let compressed = packet_compress(&original).unwrap();
	let decompressed = packet_decompress(&compressed).unwrap();
	assert_eq!(original, decompressed)
}