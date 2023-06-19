use crate::stub::{create_protocol, Channel, Data};

pub mod stub;

//
// Тестирование отправки команд с клиента на сервер
//
#[test]
fn should_send_from_client() {
	let mut peer_a = create_protocol();
	let mut peer_b = create_protocol();
	peer_a.output_data_producer.add(Data::reliable(&[1, 2, 3]));

	let mut channel = Channel::default();
	channel.cycle(1, &mut peer_a, &mut peer_b);

	assert_eq!(peer_b.input_data_handler.size_recv, 3);
}
