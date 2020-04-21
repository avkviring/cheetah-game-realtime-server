use std::rc::Rc;

use crate::relay::network::command::s2c::AffectedClients;
use crate::relay::room::clients::{Client, Clients};
use crate::relay::room::groups::AccessGroups;

#[test]
fn test_affects_client() {
	let groups = AccessGroups::from(0b111);
	let mut clients = Clients::default();
	clients.clients.insert(0, Rc::new(Client::stub_with_access_group(0, 0b1)));
	clients.clients.insert(1, Rc::new(Client::stub_with_access_group(1, 0b100000)));
	clients.clients.insert(2, Rc::new(Client::stub_with_access_group(2, 0b111)));
	
	let affected_client = AffectedClients::new(&clients, &groups);
	assert_eq!(affected_client.clients.contains(&0), true);
	assert_eq!(affected_client.clients.contains(&1), false);
	assert_eq!(affected_client.clients.contains(&2), true);
}