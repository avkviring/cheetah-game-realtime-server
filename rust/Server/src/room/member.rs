use cheetah_common::commands::CommandWithChannelType;
use cheetah_protocol::RoomMemberId;

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
