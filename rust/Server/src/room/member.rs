use cheetah_common::commands::CommandWithChannelType;
use cheetah_protocol::RoomMemberId;

use crate::room::template::config::MemberTemplate;

#[derive(Debug, Clone)]
pub struct RoomMember {
	pub id: RoomMemberId,
	pub connected: bool,
	pub attached: bool,
	pub template: MemberTemplate,
	pub out_commands: Vec<CommandWithChannelType>,
}
