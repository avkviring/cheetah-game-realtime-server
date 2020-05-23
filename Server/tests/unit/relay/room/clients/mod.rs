use cheetah_relay::network::client::ClientStream;
use cheetah_relay::network::types::hash::ToHashValue;
use cheetah_relay::room::clients::{Client, ClientConfiguration};
use cheetah_relay::room::groups::AccessGroups;
use cheetah_relay::room::objects::object::GroupType;
use cheetah_relay::room::room::ClientId;

mod room;

pub fn client_configuration_stub(client_id: ClientId) -> ClientConfiguration {
	ClientConfiguration {
		id: client_id,
		hash: format!("{}", client_id).as_str().to_hash_value(),
		groups: AccessGroups::new(),
	}
}

pub fn client_configuration_stub_with_access_group(client_id: ClientId, group: GroupType) -> ClientConfiguration {
	ClientConfiguration {
		id: client_id,
		hash: format!("{}", client_id).as_str().to_hash_value(),
		groups: AccessGroups::from(group),
	}
}

pub fn client_stub(client_id: u16) -> Client {
	Client {
		configuration: client_configuration_stub(client_id),
		stream: client_stream_stub(),
	}
}

pub fn client_stub_with_access_group(client_id: u16, groups: GroupType) -> Client {
	Client {
		configuration: client_configuration_stub_with_access_group(client_id, groups),
		stream: client_stream_stub(),
	}
}

pub fn client_stream_stub() -> ClientStream {
	ClientStream {
		stream: Option::None
	}
}
