use std::io::Cursor;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use cheetah_matches_relay_common::protocol::codec::cipher::Cipher;
use cheetah_matches_relay_common::protocol::frame::Frame;

///
/// msgpack - 1.5024 Melem/s
/// self - 1.6 Melem/s
///
fn frame_encode(c: &mut Criterion) {
	let mut group = c.benchmark_group("throughput-encode");
	group.throughput(Throughput::Elements(1));
	group.bench_function("frame_encode", |b| {
		b.iter(|| {
			let frame = Frame::new(100500);
			let mut buffer = [0; 2048];
			let private_key = [0; 32];
			frame.encode(&mut Cipher::new(&private_key), &mut buffer).unwrap();
		})
	});
	group.finish();
}

///
/// msgpack - 1.1398 Melem/s
/// self - 1.5 Melem/s
///
fn frame_decode(c: &mut Criterion) {
	let frame = Frame::new(100500);
	let mut buffer = [0; 2048];
	let private_key = [0; 32];
	let size = frame.encode(&mut Cipher::new(&private_key), &mut buffer).unwrap();

	let mut group = c.benchmark_group("throughput-decode");
	group.throughput(Throughput::Elements(1));
	group.bench_function("frame_decode", |b| {
		b.iter(|| {
			let mut cursor = Cursor::new(&buffer[0..size]);
			let (frame_id, _) = Frame::decode_headers(&mut cursor).unwrap();
			Frame::decode_frame_commands(true, frame_id, cursor, Cipher::new(&private_key)).unwrap();
		})
	});
	group.finish();
}

criterion_group!(benches, frame_encode, frame_decode);
criterion_main!(benches);
