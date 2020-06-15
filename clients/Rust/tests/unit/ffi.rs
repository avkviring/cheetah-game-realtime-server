use std::collections::HashMap;

use cheetah_relay_client::client::ffi::{FieldFFI, FieldFFIBinary, FieldsFFI, ObjectId};
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

#[test]
fn should_convert_game_object_id() {
	let owners = vec![Owner::Root, Owner::CurrentClient, Owner::Client(100)];
	for owner in owners {
		let mut ffi_game_object_id = ObjectId::default();
		let source = GameObjectId::new(100, owner);
		ffi_game_object_id.set_from(&source);
		let converted = ffi_game_object_id.to_common_game_object_id();
		assert_eq!(source, converted);
	}
}

#[test]
fn should_convert_fields_ffi() {
	let mut source = HashMap::new();
	source.insert(10 as u16, 255 as u8);
	source.insert(20 as u16, 255 as u8);
	let fields = FieldsFFI::<u8>::from(&source);
	let converted = HashMap::<u16, u8>::from(fields);
	assert_eq!(source, converted);
}

#[test]
fn should_convert_field_ffi_binary() {
	let source: Vec<u8> = vec![1, 2, 3, 4, 5];
	let field_ffi_binary = FieldFFIBinary::from(source.clone());
	let converted = Vec::from(field_ffi_binary.clone());
	assert_eq!(source, converted);
}