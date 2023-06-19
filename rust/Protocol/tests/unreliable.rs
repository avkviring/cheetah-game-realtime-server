use crate::stub::{create_protocol, Channel, Data};

pub mod stub;

///
/// Тестирование надежной доставки по ненадежному каналу
#[test]
fn should_transfer_reliable_on_unreliable_channel() {
	let mut peer_a = create_protocol();
	let mut peer_b = create_protocol();
	peer_a.output_data_producer.add(Data::reliable(&[1, 2, 3]));

	let mut channel = Channel::default();
	channel.add_reliable_percent(0..=10, 0.0);
	channel.cycle(1, &mut peer_a, &mut peer_b);

	assert_eq!(peer_b.input_data_handler.size_recv, 0);
	channel.cycle(15, &mut peer_a, &mut peer_b);
	assert_eq!(peer_b.input_data_handler.size_recv, 3);
}
