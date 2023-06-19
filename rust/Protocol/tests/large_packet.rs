use crate::stub::{create_protocol, Channel, Data};

pub mod stub;

#[test]
fn should_transfer_large_packet() {
	let mut peer_a = create_protocol();
	let mut peer_b = create_protocol();

	let original: [u8; 16384] = rand::random();
	peer_a.output_data_producer.add(Data::reliable(&original));

	let mut channel = Channel::default();
	channel.cycle(20, &mut peer_a, &mut peer_b);

	assert_eq!(peer_b.input_data_handler.items.len(), 1);
	assert_eq!(original.as_slice(), peer_b.input_data_handler.items[0].as_slice())
}
