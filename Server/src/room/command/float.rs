use crate::room::command::ServerCommandExecutor;
use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
use crate::room::{Room, User};
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::hash::UserPublicKey;

impl ServerCommandExecutor for IncrementFloat64C2SCommand {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			let value = object.fields.floats
				.entry(self.field_id)
				.and_modify(|v| *v += self.increment)
				.or_insert(self.increment)
				.clone();
			
			let access_groups = object.access_groups.clone();
			room.send(access_groups, S2CCommandUnion::SetFloat64(
				SetFloat64Command {
					object_id: self.object_id,
					field_id: self.field_id,
					value,
				}),
			);
		}
	}
}


impl ServerCommandExecutor for SetFloat64Command {
	fn execute(self, room: &mut Room, _: &UserPublicKey) {
		if let Some(object) = room.get_object(&self.object_id) {
			object.fields.floats.insert(self.field_id, self.value);
			let access_groups = object.access_groups;
			room.send(access_groups, S2CCommandUnion::SetFloat64(self));
		}
	}
}


#[cfg(test)]
mod tests {
	use std::collections::VecDeque;
	
	use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
	use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
	use cheetah_relay_common::protocol::frame::applications::ApplicationCommandChannel;
	use cheetah_relay_common::protocol::frame::Frame;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::fields::GameObjectFields;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	
	use crate::room::{Room, User};

// #[test]
	// fn test() {
	// 	let mut room = Room::new(0);
	// 	let user_public_key = 4321;
	// 	let user = User {
	// 		public_key: 0,
	// 		access_groups: Default::default(),
	// 		protocol: Default::default(),
	// 	};
	//
	// 	room.register_user(0, AccessGroups(7));
	// 	room.register_user(1, AccessGroups(7));
	// 	room.on_frame_received(&1, Frame::new(1));
	//
	//
	// 	let context = CommandContext {
	// 		current_client:
	// 		Option::Some(&user),
	// 		channel: ApplicationCommandChannel::ReliableUnordered,
	// 		meta: C2SMetaCommandInformation { timestamp: 0 },
	// 	};
	//
	//
	// 	let object_id = GameObjectId::new(10, ClientOwner::Client(user_public_key));
	// 	room.create_game_object(
	// 		&object_id,
	// 		0,
	// 		AccessGroups(55),
	// 		GameObjectFields::default(),
	// 		&context,
	// 	);
	//
	//
	// 	let field_id = 10;
	//
	//
	// 	IncrementFloat64C2SCommand {
	// 		object_id: object_id.clone(),
	// 		field_id,
	// 		increment: 10.0,
	// 	}.execute(&mut room, &context);
	//
	// 	SetFloat64Command {
	// 		object_id: object_id.clone(),
	// 		field_id,
	// 		value: 20.0,
	// 	}.execute(&mut room, &context);
	//
	// 	let mut out_frames = VecDeque::new();
	// 	room.collect_out_frames(&mut out_frames);
	// 	println!("{:?}", out_frames.into_iter().map(|m| m.frame.commands));
	// }
}