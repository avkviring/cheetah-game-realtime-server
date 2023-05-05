use cheetah_common::protocol::commands::output::CommandWithChannelType;
use cheetah_common::room::RoomMemberId;

use crate::room::template::config::MemberTemplate;

#[derive(Debug)]
pub struct RoomMember {
	pub id: RoomMemberId,
	pub connected: bool,
	pub attached: bool,
	pub template: MemberTemplate,
	pub out_commands: Vec<CommandWithChannelType>,
}
