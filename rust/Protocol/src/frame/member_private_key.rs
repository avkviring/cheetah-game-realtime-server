use std::cmp::min;

use rand::Rng;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberPrivateKey(pub [u8; 32]);

impl MemberPrivateKey {
	#[must_use]
	pub fn new_random() -> MemberPrivateKey {
		MemberPrivateKey(rand::thread_rng().gen::<[u8; 32]>())
	}
}

impl Default for MemberPrivateKey {
	fn default() -> Self {
		MemberPrivateKey::new_random()
	}
}

impl From<MemberPrivateKey> for Vec<u8> {
	fn from(source: MemberPrivateKey) -> Self {
		source.0.to_vec()
	}
}

impl From<&[u8]> for MemberPrivateKey {
	fn from(source: &[u8]) -> MemberPrivateKey {
		let mut key = [0_u8; 32];
		let len = min(source.len(), key.len());
		key[0..len].copy_from_slice(source);
		MemberPrivateKey(key)
	}
}
