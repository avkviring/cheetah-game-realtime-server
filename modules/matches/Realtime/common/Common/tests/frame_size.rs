use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
use cheetah_matches_realtime_common::commands::types::create::CreateGameObjectCommand;
use cheetah_matches_realtime_common::commands::types::field::SetFieldCommand;
use cheetah_matches_realtime_common::protocol::codec::cipher::Cipher;
use cheetah_matches_realtime_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
use cheetah_matches_realtime_common::protocol::frame::channel::Channel;
use cheetah_matches_realtime_common::protocol::frame::output::OutFrame;

#[test]
// msgpack - raw(12), compressed(5), chiper(30)
// self - raw(7), compressed(4), chiper(25)
pub fn empty_frame() {
	let frame = OutFrame::new(100500);
	let mut buffer = [0; 2048];
	let private_key = [0; 32];
	let key = private_key.as_slice().into();
	let _size = frame.encode(&mut Cipher::new(&key), &mut buffer);
	//println!("{}", size);
	// 30 байт
}

#[test]
// msgpack - raw(61), compressed(40), chiper(66)
// self - raw(21), compressed(18), chiper(39)
pub fn create_object_frame() {
	let mut frame = OutFrame::new(100500);
	frame.add_command(CommandWithChannel {
		channel: Channel::ReliableUnordered,
		both_direction_command: BothDirectionCommand::C2S(C2SCommand::CreateGameObject(CreateGameObjectCommand {
			object_id: Default::default(),
			template: 0,
			access_groups: Default::default(),
		})),
	});
	frame.add_command(CommandWithChannel {
		channel: Channel::ReliableUnordered,
		both_direction_command: BothDirectionCommand::C2S(C2SCommand::SetField(SetFieldCommand {
			object_id: Default::default(),
			field_id: 30,
			value: [0].as_ref().into(),
		})),
	});
	frame.add_command(CommandWithChannel {
		channel: Channel::ReliableUnordered,
		both_direction_command: BothDirectionCommand::C2S(C2SCommand::SetField(SetFieldCommand {
			object_id: Default::default(),
			field_id: 55,
			value: 100.into(),
		})),
	});

	let mut buffer = [0; 2048];
	let private_key = [0; 32];
	let key = private_key.as_slice().into();
	let _size = frame.encode(&mut Cipher::new(&key), &mut buffer);
	//println!("{}", size);
}
