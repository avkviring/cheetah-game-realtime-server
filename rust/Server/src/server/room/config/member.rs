use crate::server::room::config::object::GameObjectCreateParams;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectId;
use cheetah_game_realtime_protocol::frame::member_private_key::MemberPrivateKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MemberCreateParams {
	///
	/// Пользователь для которого игнорируются все настройки безопасности
	/// Обычно под данным пользователем подключаются плагины
	///
	pub super_member: bool,
	pub private_key: MemberPrivateKey,
	pub groups: AccessGroups,
	pub objects: Vec<GameObjectCreateParams>,
}

#[derive(Debug)]
pub enum MemberTemplateError {
	MemberObjectHasWrongId(MemberPrivateKey, u32),
}

impl MemberCreateParams {
	#[must_use]
	pub fn new_member(groups: AccessGroups, objects: Vec<GameObjectCreateParams>) -> Self {
		MemberCreateParams::new(false, groups, objects)
	}

	#[must_use]
	pub fn new_super_member() -> Self {
		MemberCreateParams::new(true, AccessGroups::super_member_group(), Default::default())
	}

	#[must_use]
	pub fn new_super_member_with_key(key: MemberPrivateKey) -> Self {
		let mut member = Self::new_super_member();
		member.private_key = key;
		member
	}

	fn new(super_member: bool, groups: AccessGroups, objects: Vec<GameObjectCreateParams>) -> Self {
		MemberCreateParams {
			super_member,
			private_key: MemberPrivateKey::new_random(),
			groups,
			objects,
		}
	}

	pub fn validate(self) -> Result<MemberCreateParams, MemberTemplateError> {
		for object in &self.objects {
			if object.id >= GameObjectId::CLIENT_OBJECT_ID_OFFSET {
				return Err(MemberTemplateError::MemberObjectHasWrongId(self.private_key, object.id));
			}
		}
		Ok(self)
	}
}

#[cfg(test)]
mod tests {
	use crate::server::room::config::member::{MemberCreateParams, MemberTemplateError};
	use crate::server::room::config::object::GameObjectCreateParams;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::object::{GameObjectId, GameObjectTemplateId};

	impl MemberCreateParams {
		#[must_use]
		pub fn stub(access_group: AccessGroups) -> Self {
			MemberCreateParams::new_member(access_group, Default::default())
		}

		pub fn configure_object(&mut self, id: u32, template: GameObjectTemplateId, access_groups: AccessGroups) -> &mut GameObjectCreateParams {
			let objects = &mut self.objects;
			objects.push(GameObjectCreateParams {
				id,
				template,
				groups: access_groups,
				longs: Default::default(),
				doubles: Default::default(),
				structures: Default::default(),
			});
			let len = objects.len();
			let option = objects.get_mut(len - 1);
			option.unwrap()
		}
	}

	#[test]
	fn should_validate_fail_when_member_object_has_wrong_id() {
		let objects = vec![GameObjectCreateParams {
			id: GameObjectId::CLIENT_OBJECT_ID_OFFSET + 1,
			template: 0b100,
			groups: AccessGroups(0b1111),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		}];
		let template = MemberCreateParams::new_member(AccessGroups(0b1111), objects);
		assert!(matches!(template.validate(), Err(MemberTemplateError::MemberObjectHasWrongId(_, _))));
	}
}
