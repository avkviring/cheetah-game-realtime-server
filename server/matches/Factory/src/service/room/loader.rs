use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::proto::matches::relay::types as relay;
use crate::service::error::Error;
use crate::service::room::group::{GroupAlias, GroupResolver};
use crate::service::room::prefab::PrefabResolver;
use crate::service::room::{Config, ExtendField, FieldValue, Object, ObjectField, OptionValue, Prefab, PrefabAlias, PrefabField, Room, Rule};

#[derive(Default, Debug)]
pub struct Loader {
	groups: HashMap<PathBuf, HashMap<GroupAlias, u64>>,
	prefabs: HashMap<PathBuf, Prefab>,
	rooms: HashMap<PathBuf, Room>,
}

impl Loader {
	pub fn load(dir: impl AsRef<Path>) -> Result<Self, Error> {
		Self::default().load_impl(dir.as_ref(), Path::new("/"))
	}

	fn load_impl(mut self, dir: &Path, prefix: &Path) -> Result<Self, Error> {
		let entries = dir
			.read_dir()?
			.filter_map(Result::ok)
			// пропускаем служебные каталоги при монтировании ConfigMap в kubernetes
			.filter(|entry| entry.path().to_str().map_or(false, |p| !p.contains("..")));

		for entry in entries {
			let (name, entry_type) = match (entry.file_name().into_string(), entry.file_type()) {
				(Ok(name), Ok(entry_type)) => (name, entry_type),
				_ => continue,
			};

			if entry_type.is_dir() {
				let prefix = prefix.join(name);
				self = self.load_impl(&entry.path(), &prefix)?;
			} else if let Some(name) = name.strip_suffix(".yaml").or_else(|| name.strip_suffix(".yml")) {
				let name = prefix.join(name);
				let file = std::fs::File::open(entry.path())?;
				assert!(match serde_yaml::from_reader::<_, Config>(file)? {
					Config::Room(room) => self.rooms.insert(name, room).is_none(),
					Config::Prefab(prefab) => self.prefabs.insert(name, prefab).is_none(),
					Config::Groups { groups } => self.groups.insert(name, groups).is_none(),
				});
			}
		}

		Ok(self)
	}

	pub fn resolve(self) -> Result<HashMap<String, relay::RoomTemplate>, Error> {
		let Self { groups, prefabs, rooms } = self;

		// парсим все группы
		let (local_groups, global_groups) = GroupResolver::build(groups);

		let all_prefabs: HashMap<PathBuf, PrefabResolver> = prefabs
			.into_iter()
			.map(|(path, prefab)| {
				let groups = local_groups.get(&prefab.groups).unwrap_or(&global_groups);
				PrefabResolver::new(prefab, groups, &path).map(|prefab| (path, prefab))
			})
			.collect::<Result<_, Error>>()?;

		rooms
			.into_iter()
			.map(|(path, room)| {
				log::info!("load room {:?}", path.file_name().unwrap());
				let Room { groups, prefabs, objects } = room;

				let groups = local_groups.get(&groups).unwrap_or(&global_groups);

				let objects: Vec<_> = objects
					.into_iter()
					.enumerate()
					.map(|(id, object)| -> Result<_, Error> {
						let prefab = prefabs
							.get(&object.prefab)
							.and_then(|prefab| all_prefabs.get(prefab))
							.or_else(|| all_prefabs.get(&object.prefab))
							.ok_or_else(|| Error::PrefabNotFound(path.clone(), object.prefab.clone()))?;

						let groups = groups
							.resolve_mask(&object.group)
							.ok_or_else(|| Error::GroupNotFound(path.clone(), object.group.clone()))?;

						Ok(relay::GameObjectTemplate {
							id: id as u32 + 1,
							template: prefab.template_id(),
							groups,
							fields: Some(prefab.resolve(object.fields, object.extend, &path)?),
						})
					})
					.collect::<Result<_, Error>>()?;

				let permissions = all_prefabs.values().map(|prefab| prefab.template()).collect();

				let room = relay::RoomTemplate {
					objects,
					permissions: Some(relay::Permissions { objects: permissions }),
				};

				Ok((path.display().to_string(), room))
			})
			.collect::<Result<_, Error>>()
	}
}

#[cfg(test)]
#[test]
fn loader() {
	use crate::service::test;
	use rmpv::{Integer, Utf8String, Value};

	let test_group = "test_group";
	let groups = {
		let mut groups = HashMap::default();
		groups.insert(test_group.to_string(), 4444);

		let file = serde_yaml::to_string(&Config::Groups { groups }).unwrap();
		println!("{}", file);
		file
	};

	let prefab = {
		let mut access = HashMap::default();
		access.insert(test_group.to_string(), Rule::Deny);

		let prefab = Prefab {
			template: 1234,
			groups: PathBuf::new(),
			access: access.clone(),
			fields: vec![
				PrefabField {
					name: "a".to_string(),
					id: 1,
					access: access.clone(),
					field: OptionValue::Struct {
						value: Some(Value::Map(vec![
							(Value::String(Utf8String::from("uid")), Value::String(Utf8String::from("arts80"))),
							(Value::String(Utf8String::from("rank")), Value::Integer(Integer::from(100))),
						])),
					},
				},
				PrefabField {
					name: "b".to_string(),
					id: 2,
					access: access.clone(),
					field: OptionValue::I64 { value: Some(4) },
				},
				PrefabField {
					name: "c".to_string(),
					id: 3,
					access: HashMap::default(),
					field: OptionValue::F64 { value: None },
				},
			],
		};

		let file = serde_yaml::to_string(&Config::Prefab(prefab)).unwrap();
		println!("{}", file);
		file
	};

	let room = {
		let fields = vec![ObjectField {
			name: "a".into(),
			value: FieldValue::I64 { value: 12345 },
		}];

		let extend = vec![ExtendField {
			id: 4321,
			value: FieldValue::I64 { value: 12345 },
		}];

		let mut room = Room {
			groups: "/dir/group_list".into(),
			..Room::default()
		};
		room.prefabs.insert("foo".into(), "/dir/prefab".into());
		room.objects.push(Object {
			prefab: "/dir/prefab".into(),
			group: test_group.into(),
			fields: Default::default(),
			extend: Default::default(),
		});
		room.objects.push(Object {
			prefab: "foo".into(),
			group: "test_group".into(),
			fields,
			extend,
		});

		let file = serde_yaml::to_string(&Config::Room(room)).unwrap();
		println!("{}", file);
		file
	};

	let dir = tempfile::TempDir::new().unwrap();
	let dir = {
		test::write_file_str(dir.path().join("dir/group_list.yaml"), &groups);
		test::write_file_str(dir.path().join("dir/prefab.yaml"), &prefab);
		test::write_file_str(dir.path().join("room.yaml"), &room);
		dir.path()
	};

	let loader = dbg!(Loader::load(dir).unwrap());

	let rooms = dbg!(loader.resolve().unwrap());

	{
		let room = &rooms[&"/room".to_string()];

		assert_eq!(room.objects[0].template, 1234);
		assert_eq!(room.objects[0].groups, 4444);
		assert_eq!(room.objects[0].fields.as_ref().unwrap().longs[&2], 4);

		assert_eq!(room.objects[1].fields.as_ref().unwrap().longs[&4321], 12345);

		let perm = &room.permissions.as_ref().unwrap().objects;
		assert_eq!(perm[0].template, 1234);
		assert_eq!(perm[0].rules[0].groups, 4444);
	}
}
