use std::time::Instant;

use prometheus::{Histogram, HistogramOpts, IntCounter};

use cheetah_protocol::Protocol;

use crate::stub::{Channel, StubDataRecvHandler, StubDataSource};

pub mod stub;

//
// Тестирование отправки команд с клиента на сервер
//
#[test]
fn should_send_from_client() {
	let mut peer_a = create_protocol();
	let mut peer_b = create_protocol();
	peer_a.output_data_producer.add(&[1, 2, 3]);

	let mut channel = Channel::default();
	channel.cycle(1, &mut peer_a, &mut peer_b);

	assert_eq!(peer_b.input_data_handler.size_recv, 3);
}

///
/// Тестирование надежной доставки по ненадежному каналу
#[test]
fn should_transfer_reliable_on_unreliable_channel() {
	let mut peer_a = create_protocol();
	let mut peer_b = create_protocol();
	peer_a.output_data_producer.add(&[1, 2, 3]);

	let mut channel = Channel::default();
	channel.add_reliable_percent(0..=10, 0.0);
	channel.cycle(1, &mut peer_a, &mut peer_b);

	assert_eq!(peer_b.input_data_handler.size_recv, 0);
	channel.cycle(15, &mut peer_a, &mut peer_b);
	assert_eq!(peer_b.input_data_handler.size_recv, 3);
}

fn create_protocol() -> Protocol<StubDataRecvHandler, StubDataSource> {
	Protocol::<StubDataRecvHandler, StubDataSource>::new(
		Default::default(),
		Default::default(),
		0,
		Instant::now(),
		Instant::now(),
		IntCounter::new("name", "help").unwrap().local(),
		Histogram::with_opts(HistogramOpts::new("name", "help")).unwrap().local(),
	)
}
