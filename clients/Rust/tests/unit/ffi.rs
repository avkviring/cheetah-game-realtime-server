use std::collections::HashMap;

use cheetah_relay_client::client::ffi::bytes::Bytes;
use cheetah_relay_client::client::ffi::C2SCommandFFIType::Structure;
use cheetah_relay_client::client::ffi::counters::Counters;
use cheetah_relay_client::client::ffi::ObjectId;
use cheetah_relay_client::client::ffi::structures::Structures;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

#[test]
fn should_convert_game_object_id() {
	let owners = vec![ClientOwner::Root, ClientOwner::CurrentClient, ClientOwner::Client(100)];
	for owner in owners {
		let mut ffi_game_object_id = ObjectId::default();
		let source = ClientGameObjectId::new(100, owner);
		ffi_game_object_id.set_from(&source);
		let converted = ffi_game_object_id.to_common_game_object_id();
		assert_eq!(source, converted);
	}
}

#[test]
fn should_convert_counters() {
	let mut source = HashMap::new();
	source.insert(10 as u16, 255 as u8);
	source.insert(20 as u16, 255 as u8);
	let fields = Counters::<u8>::from(&source);
	let converted = HashMap::<u16, u8>::from(&fields);
	assert_eq!(source, converted);
}

#[test]
fn should_convert_bytes() {
	let source: Vec<u8> = vec![1, 2, 3, 4, 5];
	let field_ffi_binary = Bytes::from(source.clone());
	let converted = Vec::from(field_ffi_binary.clone());
	assert_eq!(source, converted);
}


#[test]
fn should_convert_structures() {
	let mut source = HashMap::<FieldID, Vec<u8>>::new();
	source.insert(10, vec![1, 2, 3, 4, 5]);
	source.insert(20, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
	
	let structures = Structures::from(&source);
	let converted = HashMap::<FieldID, Vec<u8>>::from(&structures);
	
	assert_eq!(source, converted);
}