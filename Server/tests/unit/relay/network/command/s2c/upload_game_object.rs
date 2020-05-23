use std::collections::HashMap;

use cheetah_relay::network::command::s2c::S2CCommand;
use cheetah_relay::network::command::s2c::upload_object::UploadGameObjectS2CCommand;
use cheetah_relay::room::groups::AccessGroups;
use cheetah_relay::room::objects::object::{DataStruct, FloatCounter, GameObject, LongCounter};
use cheetah_relay::room::objects::owner::Owner;
use crate::unit::relay::network::command::s2c::create_buffer_with_capacity;

#[test]
fn should_true_when_buffer_is_enough() {
	let mut buffer = create_buffer_with_capacity(SIZE_ID + SIZE_COUNT + SIZE_LONG_COUNTER + SIZE_FLOAT_COUNTER + SIZE_STRUCTURES);
	assert_eq!(create_command().encode(&mut buffer), true)
}

#[test]
fn should_false_when_buffer_for_write_is_small_1() {
	let mut buffer = create_buffer_with_capacity(0);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_2() {
	let mut buffer = create_buffer_with_capacity(SIZE_ID);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_3() {
	let mut buffer = create_buffer_with_capacity(SIZE_ID + SIZE_COUNT);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_4() {
	let mut buffer = create_buffer_with_capacity(SIZE_ID + SIZE_COUNT + SIZE_LONG_COUNTER);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_5() {
	let mut buffer = create_buffer_with_capacity(SIZE_ID + SIZE_COUNT + SIZE_LONG_COUNTER + SIZE_FLOAT_COUNTER);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_6() {
	let mut buffer = create_buffer_with_capacity(SIZE_ID + SIZE_COUNT + SIZE_LONG_COUNTER + SIZE_FLOAT_COUNTER + SIZE_STRUCTURES - 1);
	assert_eq!(create_command().encode(&mut buffer), false)
}


#[test]
fn should_correct_write() {
	let mut buffer = create_buffer_with_capacity(1024);
	let command = create_command();
	command.encode(&mut buffer);
	buffer.flip();
	let object_id = buffer.read_u64().unwrap();
	let long_counter_count = buffer.read_u16().unwrap();
	let float_counter_count = buffer.read_u16().unwrap();
	let structures_count = buffer.read_u16().unwrap();
	
	let mut long_counters = Vec::new();
	for _ in 0..long_counter_count {
		let field_id = buffer.read_u16().unwrap();
		let value = buffer.read_i64().unwrap();
		long_counters.push((field_id, LongCounter { counter: value }));
	}
	
	let mut float_counters = Vec::new();
	for _ in 0..float_counter_count {
		let field_id = buffer.read_u16().unwrap();
		let value = buffer.read_f64().unwrap();
		float_counters.push((field_id, FloatCounter { counter: value }));
	}
	
	let mut structures = Vec::new();
	for _ in 0..structures_count {
		let field_id = buffer.read_u16().unwrap();
		let size = buffer.read_u16().unwrap() as usize;
		let data = buffer.read_to_vec(size).unwrap();
		structures.push((field_id, DataStruct { data }));
	}
	
	
	let object = GameObject {
		id: object_id,
		owner: Owner::new_root_owner(),
		structures: structures.iter().cloned().collect(),
		long_counters: long_counters.iter().cloned().collect(),
		float_counters: float_counters.iter().cloned().collect(),
		groups: AccessGroups::new(),
	};
	
	assert_eq!(command.cloned_object, object);
}

const SIZE_ID: usize = 8;
const SIZE_COUNT: usize = 2;
const SIZE_STRUCTURES: usize = 2 + 2 + 5;
const SIZE_LONG_COUNTER: usize = 2 + 2 + 8 + 2 + 8;
const SIZE_FLOAT_COUNTER: usize = 2 + 2 + 8 + 2 + 8 + 2 + 8;

fn create_command() -> UploadGameObjectS2CCommand {
	UploadGameObjectS2CCommand {
		cloned_object: GameObject {
			id: 123123,
			owner: Owner::new_root_owner(),
			structures: [(10, DataStruct { data: vec![1, 2, 3, 4, 5] })].iter().cloned().collect(),
			long_counters: [
				(20, LongCounter { counter: 100_500 }),
				(30, LongCounter { counter: 100_501 })].iter().cloned().collect(),
			float_counters: [
				(40, FloatCounter { counter: 100_500.0 }),
				(50, FloatCounter { counter: 100_501.0 }),
				(60, FloatCounter { counter: 100_502.0 })].iter().cloned().collect(),
			groups: AccessGroups::new(),
		}
	}
}