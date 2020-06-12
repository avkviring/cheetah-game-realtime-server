use cheetah_relay::network::client::ClientStream;
use cheetah_relay::room::clients::{Client, ClientConfiguration};
use cheetah_relay_common::constants::{ClientId, GroupType};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::network::hash::HashValue;

mod room;

pub fn client_configuration_stub(client_id: ClientId) -> ClientConfiguration {
    ClientConfiguration {
        id: client_id,
        hash: HashValue::from(format!("{}", client_id).as_str()),
        groups: AccessGroups::default(),
    }
}

pub fn client_configuration_stub_with_access_group(client_id: ClientId, group: GroupType) -> ClientConfiguration {
    ClientConfiguration {
        id: client_id,
        hash: HashValue::from(format!("{}", client_id).as_str()),
        groups: AccessGroups::from(group),
    }
}

pub fn client_stub(client_id: u16) -> Client {
    Client {
        configuration: client_configuration_stub(client_id),
    }
}

pub fn client_stub_with_access_group(client_id: u16, groups: GroupType) -> Client {
    Client {
        configuration: client_configuration_stub_with_access_group(client_id, groups)
    }
}

pub fn client_stream_stub() -> ClientStream {
    ClientStream {
        stream: Option::None
    }
}
