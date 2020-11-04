use std::cell::RefCell;
use std::rc::Rc;
use std::task::Context;

use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;

use crate::network::s2c::S2CCommandCollector;
use crate::room::command::CommandContext;
use crate::room::object::id::ServerGameObjectId;

pub mod id;


///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone)]
pub struct GameObject {
	pub id: ServerGameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
	collector: Rc<RefCell<S2CCommandCollector>>,
}


impl GameObject {
	pub fn new(id: ServerGameObjectId, template: u16, access_groups: AccessGroups, fields: GameObjectFields, collector: Rc<RefCell<S2CCommandCollector>>) -> GameObject {
		GameObject {
			id,
			template,
			access_groups,
			fields,
			collector,
		}
	}
	
	pub fn send_to_clients<F: FnMut(&ClientId, ClientGameObjectId) -> S2CCommandUnion>(&mut self, context: &CommandContext, mut factory: F) {
		let mut collector = self.collector.borrow_mut();
		collector.collect(self, context, |client_id| {
			factory(client_id, self.id.to_client_object_id(Some(*client_id)))
		});
	}
}


