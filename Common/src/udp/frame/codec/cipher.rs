//use sodiumoxide::crypto::aead::chacha20poly1305::{Key, Nonce, open, seal};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead, Payload};

use crate::network::hash::{UserPrivateKey, UserPublicKey};

///
/// Шифрование пакета
///
/// - проверка правильности дешифрации
/// - проверка открытых данных (aead)
///
///
#[derive(Clone)]
pub struct Cipher {
	nonce: [u8; 12],
	private_key: UserPrivateKey,
}

impl Cipher {
	pub fn new(private_key: UserPrivateKey) -> Self {
		Self {
			nonce: Default::default(),
			private_key,
		}
	}
	
	pub fn encrypt(&mut self, msg: &[u8], ad: &[u8], nonce: [u8; 8]) -> Vec<u8> {
		self.nonce[0..8].copy_from_slice(&nonce);
		let key = Key::from_slice(&self.private_key);
		let nonce = Nonce::from_slice(&self.nonce);
		let cipher = ChaCha20Poly1305::new(key);
		cipher.encrypt(nonce, Payload {
			msg,
			aad: ad,
		}).unwrap()
	}
	
	pub fn decrypt(&mut self, msg: &[u8], ad: &[u8], nonce: [u8; 8]) -> Result<Vec<u8>, ()> {
		self.nonce[0..8].copy_from_slice(&nonce);
		let key = Key::from_slice(&self.private_key);
		let nonce = Nonce::from_slice(&self.nonce);
		let cipher = ChaCha20Poly1305::new(key);
		cipher.decrypt(nonce, Payload {
			msg,
			aad: ad,
		}).map_err(|_| ())
	}
}

