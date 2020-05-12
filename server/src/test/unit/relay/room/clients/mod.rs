use crate::relay::room::clients::{ClientConfiguration, Client};
use crate::relay::room::room::ClientId;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::object::GroupType;
use crate::relay::network::client::ClientStream;
use crate::relay::network::types::hash::ToHashValue;

mod room;

impl ClientConfiguration {
	fn stub(client_id: ClientId) -> ClientConfiguration {
		ClientConfiguration {
			id: client_id,
			hash: format!("{}", client_id).as_str().to_hash_value(),
			groups: AccessGroups::new(),
		}
	}
	
	fn stub_with_access_group(client_id: ClientId, group: GroupType) -> ClientConfiguration {
		ClientConfiguration {
			id: client_id,
			hash: format!("{}", client_id).as_str().to_hash_value(),
			groups: AccessGroups::from(group),
		}
	}
}

impl Client {
	pub fn stub(client_id: u16) -> Client {
		Client {
			configuration: ClientConfiguration::stub(client_id),
			stream: ClientStream::stub(),
		}
	}
	pub fn stub_with_access_group(client_id: u16, groups: GroupType) -> Client {
		Client {
			configuration: ClientConfiguration::stub_with_access_group(client_id, groups),
			stream: ClientStream::stub(),
		}
	}
}


impl ClientStream {
	pub fn stub() -> ClientStream {
		ClientStream {
			stream: Option::None
		}
	}
}
