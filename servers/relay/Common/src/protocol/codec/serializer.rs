use std::io::Cursor;

use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

pub fn serialize<T: Serialize>(item: T, out: &mut Cursor<&mut [u8]>) {
	item.serialize(&mut Serializer::new(out)).unwrap();
}

pub fn deserialize<'a, T: Deserialize<'a>>(input: &mut Cursor<&[u8]>) -> Result<T, ()> {
	let mut de = rmp_serde::Deserializer::new(input);
	Deserialize::deserialize(&mut de).map_err(|_| ())
}
