use crate::server::room::config::member::MemberCreateParams;
use cheetah_common::commands::CommandWithChannelType;
use cheetah_game_realtime_protocol::RoomMemberId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMember {
	pub id: RoomMemberId,
	pub status: RoomMemberStatus,
	pub template: MemberCreateParams,
	pub out_commands: Vec<CommandWithChannelType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RoomMemberStatus {
	Created,
	CreatedNotConnectedAndDeleted,
	Connected,
	Attached,
	Detached,
	Disconnected,
}

impl RoomMemberStatus {
	pub fn is_online(&self) -> bool {
		return *self == Self::Connected || *self == Self::Attached || *self == Self::Detached;
	}
}
