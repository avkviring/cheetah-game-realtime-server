use std::ops::{Div, Rem};

use easybench::bench;
use rand::RngCore;
use rand::rngs::OsRng;

use cheetah_relay_common::udp::transport::crypto::Crypto;

#[test]
fn bench_crypto() {
	let secret_key = [
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9,
		0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05,
		0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f,
		0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];
	let original = vec![0 as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
	let nonce = [0; 12];
	let aad = vec![0, 1, 2, 3];
	let crypto = Crypto::new(secret_key);
	println!("{}", bench(|| {
		let mut buffer = original.clone();
		crypto.encrypt(&mut buffer, &aad, nonce);
		crypto.decrypt(&mut buffer, &*aad, nonce);
	}))
}

#[test]
fn should_crypto() {
	// let secret_key = [
	// 	0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9,
	// 	0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05,
	// 	0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f,
	// 	0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	// ];
	// let crypto = Crypto::new(secret_key);
	// let original = vec![0 as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
	// let nonce = [0; 12];
	// let aad = vec![0, 1, 2, 3];
	//
	// let encrypted = crypto.encrypt(&*original, &aad, nonce);
	// assert_ne!(encrypted, original);
	// let decrypted = crypto.decrypt(&*encrypted, &*aad, nonce);
	// assert_eq!(decrypted, original)
}
