use crate::frame::member_private_key::MemberPrivateKey;
use chacha20poly1305::aead::{AeadInPlace, Error};
use chacha20poly1305::{ChaCha8Poly1305, Key, KeyInit, Nonce};
use heapless::Vec;

///
/// Шифрование пакета
///
/// - проверка правильности дешифрации
/// - проверка открытых данных (aead)
///
///
#[derive(Clone)]
pub struct Cipher<'a> {
	private_key: &'a MemberPrivateKey,
}

impl<'a> Cipher<'a> {
	#[must_use]
	pub fn new(private_key: &'a MemberPrivateKey) -> Self {
		Self { private_key }
	}

	pub fn encrypt(&mut self, buffer: &mut Vec<u8, 4096>, ad: &[u8], nonce: [u8; 8]) -> Result<(), Error> {
		let mut nonce_buffer = [0; 12];
		nonce_buffer[0..8].copy_from_slice(&nonce);
		let key = Key::from_slice(&self.private_key.0);
		let nonce = Nonce::from_slice(&nonce_buffer);
		let cipher = ChaCha8Poly1305::new(key);
		cipher.encrypt_in_place(nonce, ad, buffer)?;
		Ok(())
	}

	pub fn decrypt(&mut self, buffer: &mut Vec<u8, 4096>, ad: &[u8], nonce: [u8; 8]) -> Result<(), Error> {
		let mut nonce_buffer = [0; 12];
		nonce_buffer[0..8].copy_from_slice(&nonce);
		let key = Key::from_slice(&self.private_key.0);
		let nonce = Nonce::from_slice(&nonce_buffer);
		let cipher = ChaCha8Poly1305::new(key);
		cipher.decrypt_in_place(nonce, ad, buffer)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use heapless::Vec;

	use crate::codec::cipher::Cipher;

	const PRIVATE_KEY: &[u8] = &[
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9, 0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05, 0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f, 0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];
	const ORIGINAL: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
	const NONCE: [u8; 8] = [0; 8];
	const AD: [u8; 4] = [1, 2, 3, 4];
	const OTHER_AD: [u8; 2] = [0, 1];

	#[test]
	fn should_cipher() {
		let key = PRIVATE_KEY.into();
		let mut cipher = Cipher::new(&key);
		let mut buffer: Vec<u8, 4096> = Vec::new();
		buffer.extend_from_slice(&ORIGINAL).unwrap();
		cipher.encrypt(&mut buffer, &AD, NONCE).unwrap();
		assert_ne!(&buffer, &ORIGINAL);
		cipher.decrypt(&mut buffer, &AD, NONCE).unwrap();
		assert_eq!(&buffer, &ORIGINAL);
	}

	#[test]
	fn should_fail_when_different_ad() {
		let key = PRIVATE_KEY.into();
		let mut cipher = Cipher::new(&key);
		let mut buffer: Vec<u8, 4096> = Vec::new();
		buffer.extend_from_slice(&ORIGINAL).unwrap();
		cipher.encrypt(&mut buffer, &AD, NONCE).unwrap();

		assert!(matches!(cipher.decrypt(&mut buffer, &OTHER_AD, NONCE), Err(_)));
	}

	#[test]
	fn should_fail_when_broken_packet() {
		let key = PRIVATE_KEY.into();
		let mut cipher = Cipher::new(&key);
		let mut buffer: Vec<u8, 4096> = Vec::new();
		buffer.extend_from_slice(&ORIGINAL).unwrap();
		cipher.encrypt(&mut buffer, &AD, NONCE).unwrap();
		buffer[0] = 0;
		assert!(matches!(cipher.decrypt(&mut buffer, &AD, NONCE), Err(_)));
	}
}
