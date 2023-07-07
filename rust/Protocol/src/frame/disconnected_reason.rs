use crate::disconnect::command::DisconnectByCommandReason;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DisconnectedReason {
	IOError(String),
	Timeout,
	Command(DisconnectByCommandReason),
	RetransmitOverflow,
}
