use crate::disconnect::command::DisconnectByCommandReason;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DisconnectedReason {
	IOError(String),
	ByTimeout,
	ByCommand(DisconnectByCommandReason),
	ByRetransmitWhenMaxCount,
	ByRetransmitWhenMaxFrames,
	ByRetransmitWhenMaxWaitAck,
}
