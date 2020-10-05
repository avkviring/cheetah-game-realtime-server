use cheetah_relay_common::udp::frame::codec::cipher::Cipher;
use cheetah_relay_common::udp::frame::format::{ApplicationCommand, AskFrameUdpItem, UdpAdditionalHeader, UdpFrame, UdpFrameHeader};

pub mod cipher;
pub mod compress;

#[test]
fn should_encode_decode_frame() {
	let mut frame = UdpFrame::new(0);
	let mut cipher = Cipher::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
	frame.additional_headers.push(UdpAdditionalHeader::ASK(AskFrameUdpItem { ask_packet_id: 10 }));
	frame.additional_headers.push(UdpAdditionalHeader::ASK(AskFrameUdpItem { ask_packet_id: 15 }));
	frame.commands.push(ApplicationCommand::Ping("test".to_string()));
	let binary_frame = frame.encode(&mut cipher);
	let decoded_frame = UdpFrame::decode(|_| { Result::Ok(cipher.clone()) }, &binary_frame.0).ok().unwrap();
	assert_eq!(frame, decoded_frame);
}


#[test]
fn should_limit_buffer_size() {
	let mut frame = UdpFrame::new(0);
	let mut cipher = Cipher::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
	const command_count: usize = 400;
	for _ in 0..command_count {
		frame.commands.push(ApplicationCommand::Ping("1234567890".to_string()));
	}
	let binary_frame = frame.encode(&mut cipher);
	let decoded_frame = UdpFrame::decode(|_| { Result::Ok(cipher.clone()) }, &binary_frame.0).ok().unwrap();
	assert!(binary_frame.0.len() <= UdpFrame::MAX_FRAME_SIZE);
	assert!(binary_frame.1.len() + frame.commands.len() == command_count);
	assert_eq!(frame, decoded_frame);
}
