use std::time::Instant;

use crate::stub::{create_protocol, Channel, Data};

pub mod stub;

//
// Тестирование отправки команд с клиента на сервер
//
#[test]
fn stress_test() {
	let mut peer_a = create_protocol();
	let mut peer_b = create_protocol();

	let mut channel = Channel::default();
	channel.add_reliable_percent(0..=1000, 0.999999);
	for _ in 0..100000 {
		let original: [u8; 10] = rand::random();
		peer_a.output_data_producer.add(Data::reliable(&original));
		peer_b.output_data_producer.add(Data::reliable(&original));
		channel.cycle(1, &mut peer_a, &mut peer_b);
		assert!(peer_a.is_connected(Instant::now()));
		assert!(peer_a.is_connected(Instant::now()));
	}
}
