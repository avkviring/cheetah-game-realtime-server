use std::io::{Cursor, ErrorKind};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::disconnect::command::DisconnectHeader;
use crate::frame::headers::{Header, HeaderVec, Headers};
use crate::others::member_id::MemberAndRoomId;
use crate::others::rtt::RoundTripTimeHeader;
use crate::reliable::ack::header::AckHeader;
use crate::reliable::retransmit::header::RetransmitHeader;

impl Headers {
	#[allow(clippy::map_err_ignore)]
	pub fn decode_headers(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let mut headers = HeaderVec::new();
		let count = input.read_variable_u64()?;
		for _ in 0..count {
			let type_header = input.read_u8()?;
			let header = match type_header {
				0 => Header::MemberAndRoomId(MemberAndRoomId::decode(input)?),
				1 => Header::Ack(AckHeader::decode(input)?),
				2 => Header::Disconnect(DisconnectHeader::decode(input)?),
				3 => Header::RoundTripTimeRequest(RoundTripTimeHeader::decode(input)?),
				4 => Header::RoundTripTimeResponse(RoundTripTimeHeader::decode(input)?),
				5 => Header::Retransmit(RetransmitHeader::decode(input)?),
				6 => Header::Hello,
				_ => {
					return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Invalid type header {type_header}")));
				}
			};
			headers.push(header).map_err(|_| std::io::Error::new(ErrorKind::InvalidInput, "Headers count overflow"))?;
		}
		Ok(Self { headers })
	}

	pub fn encode_headers(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.headers.len() as u64)?;
		for header in &self.headers {
			match header {
				Header::MemberAndRoomId(data) => {
					out.write_u8(0)?;
					data.encode(out)?;
				}
				Header::Ack(data) => {
					out.write_u8(1)?;
					data.encode(out)?;
				}
				Header::Disconnect(data) => {
					out.write_u8(2)?;
					data.encode(out)?;
				}
				Header::RoundTripTimeRequest(data) => {
					out.write_u8(3)?;
					data.encode(out)?;
				}
				Header::RoundTripTimeResponse(data) => {
					out.write_u8(4)?;
					data.encode(out)?;
				}
				Header::Retransmit(data) => {
					out.write_u8(5)?;
					data.encode(out)?;
				}
				Header::Hello => {
					out.write_u8(6)?;
				}
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::disconnect::command::{DisconnectByCommandReason, DisconnectHeader};
	use crate::frame::headers::{Header, HeaderVec, Headers};
	use crate::others::member_id::MemberAndRoomId;
	use crate::others::rtt::RoundTripTimeHeader;
	use crate::reliable::ack::header::AckHeader;
	use crate::reliable::retransmit::header::RetransmitHeader;

	#[test]
	fn test_hello() {
		check(&[Header::Hello]);
	}

	#[test]
	fn test_member_and_room() {
		check(&[Header::MemberAndRoomId(MemberAndRoomId { member_id: 55, room_id: 77 })]);
	}

	#[test]
	fn test_disconnect() {
		check(&[Header::Disconnect(DisconnectHeader(DisconnectByCommandReason::RoomDeleted))]);
	}

	#[test]
	fn test_retransmit_frame() {
		check(&[Header::Retransmit(RetransmitHeader { original_frame_id: 100 })]);
	}

	#[test]
	fn test_ack() {
		check(&[Header::Ack(AckHeader::default())]);
	}

	#[test]
	fn test_rtt_response() {
		check(&[Header::RoundTripTimeResponse(RoundTripTimeHeader { self_time: 155 })]);
	}

	#[test]
	fn test_rtt_request() {
		check(&[Header::RoundTripTimeRequest(RoundTripTimeHeader { self_time: 155 })]);
	}

	fn check(headers: &[Header]) {
		let headers = HeaderVec::from_slice(headers).unwrap();
		let headers = Headers { headers };
		let mut data = [0_u8; 100];
		let mut out = Cursor::new(data.as_mut());
		Headers::encode_headers(&headers, &mut out).unwrap();
		let write_position = out.position();
		let mut input = Cursor::<&[u8]>::new(&data);
		let actual = Headers::decode_headers(&mut input).unwrap();
		assert_eq!(write_position, input.position());
		assert_eq!(headers, actual);
	}
}
