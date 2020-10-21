use rand::rngs::OsRng;

use cheetah_relay_common::commands::hash::{UserPrivateKey, UserPublicKey};
use rand::RngCore;

pub mod protocol;

pub fn create_user_private_key_stub() -> UserPrivateKey {
	let mut result = [0; 32];
	OsRng.fill_bytes(&mut result);
	result
}


pub fn create_user_public_key_stub() -> UserPublicKey {
	let mut result = [0; 4];
	OsRng.fill_bytes(&mut result);
	result
}


