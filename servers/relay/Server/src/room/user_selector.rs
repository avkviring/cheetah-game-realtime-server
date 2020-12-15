use std::collections::HashMap;
use std::ops::Add;
use std::time::{Duration, Instant};

use fnv::FnvBuildHasher;

use serde::{Deserialize, Serialize};

use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};

use crate::room::Room;

///
/// Выбор пользователя для входа
/// в режиме работы сервера без ММ
///
#[derive(Debug, Default)]
pub struct UserForEntranceSelector {
	selected: HashMap<UserPublicKey, Instant, FnvBuildHasher>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectedUserForEntrance {
	pub public_key: UserPublicKey,
	pub private_key: UserPrivateKey,
}

impl UserForEntranceSelector {
	const SELECT_TIMEOUT: Duration = Duration::from_secs(5);

	pub fn select(&mut self, room: &Room) -> Option<SelectedUserForEntrance> {
		self.do_select(room, &Instant::now())
	}

	fn do_select(&mut self, room: &Room, now: &Instant) -> Option<SelectedUserForEntrance> {
		self.selected.retain(|_key, time| *time > *now);

		let result = room
			.users
			.iter()
			.filter(|(_key, user)| user.protocol.is_none())
			.filter(|(key, _user)| !self.selected.contains_key(key))
			.find_map(|(key, user)| {
				Option::Some(SelectedUserForEntrance {
					public_key: key.clone(),
					private_key: user.template.private_key.clone(),
				})
			});
		match &result {
			None => {}
			Some(user) => {
				self.selected
					.insert(user.public_key.clone(), now.clone().add(UserForEntranceSelector::SELECT_TIMEOUT));
			}
		}
		result
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};

	use cheetah_relay_common::protocol::relay::RelayProtocol;
	use cheetah_relay_common::room::access::AccessGroups;

	use crate::room::template::RoomTemplate;
	use crate::room::user_selector::UserForEntranceSelector;
	use crate::room::Room;

	#[test]
	pub fn should_select_users_with_timeout() {
		let mut template = RoomTemplate::default();
		template.create_user(1, AccessGroups(1));
		template.create_user(2, AccessGroups(1));
		let room = Room::new_with_template(template);

		let mut selector = UserForEntranceSelector::default();

		let mut now = Instant::now();

		let user_1 = selector.do_select(&room, &now).unwrap();
		let user_2 = selector.do_select(&room, &now.add(Duration::from_secs(1))).unwrap();
		assert_ne!(user_1.public_key, user_2.public_key);
		assert!(selector.do_select(&room, &now).is_none());
		// после паузы - только первый пользователь должен освободится
		now = now.add(UserForEntranceSelector::SELECT_TIMEOUT);
		let unselected_user_1 = selector.do_select(&room, &now).unwrap();
		assert_eq!(user_1.public_key, unselected_user_1.public_key);
		assert!(selector.do_select(&room, &now).is_none());
	}

	#[test]
	pub fn should_dont_select_connected_users() {
		let mut template = RoomTemplate::default();
		template.create_user(1, AccessGroups(1));
		template.create_user(2, AccessGroups(1));

		let mut room = Room::new_with_template(template);
		let now = Instant::now();
		room.users.get_mut(&2).unwrap().protocol = Option::Some(RelayProtocol::new(&now));
		let mut selector = UserForEntranceSelector::default();
		let user_1 = selector.do_select(&room, &now).unwrap();

		assert_eq!(user_1.public_key, 1);
		assert!(selector.do_select(&room, &now).is_none());
	}
}
