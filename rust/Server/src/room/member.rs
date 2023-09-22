use cheetah_game_realtime_protocol::RoomMemberId;
use cheetah_common::commands::CommandWithChannelType;
use crate::room::template::config::MemberTemplate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMember {
	pub id: RoomMemberId,
	pub connected: bool,
	pub attached: bool,
	pub template: MemberTemplate,
	pub out_commands: Vec<CommandWithChannelType>,
}
