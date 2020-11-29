#![feature(test)]
extern crate test;

use test::Bencher;

use cheetah_relay_common::protocol::codec::cipher::Cipher;

use cheetah_relay_common::protocol::codec::compress::{packet_compress, packet_decompress};

const PRIVATE_KEY: &[u8; 32] = &[
	0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9,
	0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05,
	0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f,
	0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
];

const ORIGINAL: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const NONCE: [u8; 8] = [0; 8];
const AD: [u8; 4] = [1, 2, 3, 4];


#[bench]
fn bench_chiper(b: &mut Bencher) {
	b.iter(|| {
		let mut cipher = Cipher::new(PRIVATE_KEY);
		let mut buffer: heapless::Vec<u8, heapless::consts::U2048> = heapless::Vec::new();
		buffer.extend_from_slice(&ORIGINAL).unwrap();
		cipher.encrypt(&mut buffer, &AD, NONCE).unwrap();
		cipher.decrypt(&mut buffer, &AD, NONCE).unwrap();
	});
}

#[bench]
fn bench_compress(b: &mut Bencher) {
	b.iter(|| {
		let original = vec![1, 2, 3, 4, 4, 3, 2, 4, 5, 6, 7, 5, 4, 3, 4, 5, 7, 7, 8, 5, 4, 2, 3, 4, 5, 6, 7, 8];
		let mut compressed = [0; 100];
		let compressed_size = packet_compress(&original, &mut compressed).unwrap();
		let mut decompressed = [0; 100];
		packet_decompress(&compressed[0..compressed_size], &mut decompressed);
	});
}