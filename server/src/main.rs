extern crate stderrlog;
use stderrlog::Timestamp;

mod relay;

#[cfg(test)]
mod test;

fn main() {
	init_logger();
	
	
	// let listener = TcpListener::bind("127.0.0.1:5555").unwrap();
	// for stream in listener.incoming() {
	// 	print!("income");
	// 	let mut tcp_stream = stream.unwrap();
	// 	tcp_stream.write(&[65 as u8, 66 as u8, 10 as u8]);
	// }
}

fn init_logger() {
	stderrlog::new()
		.verbosity(4)
		.quiet(false)
		.timestamp(Timestamp::Millisecond)
		.init()
		.unwrap();
}