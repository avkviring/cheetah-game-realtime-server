mod group;
mod prefab;

use self::group::{GroupAlias, GroupResolver};
use self::prefab::PrefabResolver;
use crate::proto::matches::relay::types as relay;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub type FieldAlias = String;
pub type PrefabAlias = PathBuf;

#[derive(Debug)]
pub enum Error {
    PrefabNotFound(PathBuf, PrefabAlias),
    GroupNotFound(PathBuf, GroupAlias),

    PrefabFieldNotExists(PathBuf, String),
    ObjectFieldExists(PathBuf, u32),

    Io(std::io::Error),
    Yaml(serde_yaml::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PrefabNotFound(path, prefab) => {
                write!(
                    f,
                    "{}: Prefab {} not found",
                    path.display(),
                    prefab.display(),
                )
            }
            Error::GroupNotFound(path, group) => {
                write!(f, "{}: Group {} not found", path.display(), group)
            }
            Error::ObjectFieldExists(path, id) => {
                write!(f, "{}: Field {} exists", path.display(), id)
            }
            Error::PrefabFieldNotExists(path, name) => {
                write!(f, "{}: Field {} not found in prefab", path.display(), name)
            }
            Error::Io(err) => write!(f, "IO: {:?}", err),
            Error::Yaml(err) => write!(
                f,
                "Wrong file format {:?}: {:?}",
                err.location().map(|loc| (loc.line(), loc.column())),
                err
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Config {
    #[serde(rename = "room")]
    Room(Room),
    #[serde(rename = "prefab")]
    Prefab(Prefab),
    #[serde(rename = "groups")]
    Groups {
        #[serde(flatten)]
        groups: HashMap<GroupAlias, u64>,
    },
}

fn skip_path(path: &Path) -> bool {
    path.as_os_str().is_empty()
}

/// Описание комнаты
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Room {
    /// Путь до файла со всеми группами
    #[serde(default, skip_serializing_if = "skip_path")]
    pub groups: PathBuf,
    /// Шаблоны для создания объектов
    pub prefabs: HashMap<PrefabAlias, PathBuf>,
    /// Объекты комнаты
    pub objects: Vec<Object>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Prefab {
    pub template: u32,
    /// Путь до файла со всеми группами
    #[serde(default, skip_serializing_if = "skip_path")]
    pub groups: PathBuf,
    /// Права доступа для всего объекта
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub access: HashMap<GroupAlias, Rule>,
    /// Права доступа и настройки по умолчанию для объектов
    pub fields: Vec<PrefabField>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Rule {
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "ro")]
    ReadOnly,
    #[serde(rename = "rw")]
    ReadWrite,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrefabField {
    pub name: FieldAlias,
    pub id: u32,

    #[serde(flatten)]
    pub field: OptionValue,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub access: HashMap<GroupAlias, Rule>,
}

/// Описание объекта в комнате
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Object {
    /// Имя префаба
    pub prefab: PrefabAlias,
    /// Имя группы
    pub group: GroupAlias,

    /// Поля объекта
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<ObjectField>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extend: Vec<ExtendField>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectField {
    /// Имя поля из префаба
    pub name: FieldAlias,
    #[serde(flatten)]
    pub value: FieldValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtendField {
    pub id: u32,
    #[serde(flatten)]
    pub value: FieldValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "lowercase")]
pub enum FieldValue {
    I64 { value: i64 },
    F64 { value: f64 },
    Struct { value: rmpv::Value },
    Event,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "lowercase")]
pub enum OptionValue {
    I64 {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<i64>,
    },
    F64 {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<f64>,
    },
    Struct {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<rmpv::Value>,
    },
    Event,
}

impl OptionValue {
    fn into_value(self) -> Option<FieldValue> {
        Some(match self {
            OptionValue::Struct { value: Some(value) } => FieldValue::Struct { value },
            OptionValue::I64 { value: Some(value) } => FieldValue::I64 { value },
            OptionValue::F64 { value: Some(value) } => FieldValue::F64 { value },
            OptionValue::Event => FieldValue::Event,
            _ => return None,
        })
    }
}

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
            } else if let Some(name) = name
                .strip_suffix(".yaml")
                .or_else(|| name.strip_suffix(".yml"))
            {
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
        let Self {
            groups,
            prefabs,
            rooms,
        } = self;

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
                let Room {
                    groups,
                    prefabs,
                    objects,
                } = room;

                let groups = local_groups.get(&groups).unwrap_or(&global_groups);

                let objects: Vec<_> = objects
                    .into_iter()
                    .enumerate()
                    .map(|(id, object)| -> Result<_, Error> {
                        let prefab = prefabs
                            .get(&object.prefab)
                            .and_then(|prefab| all_prefabs.get(prefab))
                            .or_else(|| all_prefabs.get(&object.prefab))
                            .ok_or_else(|| {
                                Error::PrefabNotFound(path.clone(), object.prefab.clone())
                            })?;

                        let groups = groups.resolve_mask(&object.group).ok_or_else(|| {
                            Error::GroupNotFound(path.clone(), object.group.clone())
                        })?;

                        Ok(relay::GameObjectTemplate {
                            id: id as u32,
                            template: prefab.template_id(),
                            groups,
                            fields: Some(prefab.resolve(object.fields, object.extend, &path)?),
                        })
                    })
                    .collect::<Result<_, Error>>()?;

                let permissions = all_prefabs
                    .values()
                    .map(|prefab| prefab.template())
                    .collect();

                let room = relay::RoomTemplate {
                    objects,
                    permissions: Some(relay::Permissions {
                        objects: permissions,
                    }),
                };

                Ok((path.display().to_string(), room))
            })
            .collect::<Result<_, Error>>()
    }
}

#[cfg(test)]
#[test]
fn loader() {
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
                            (
                                Value::String(Utf8String::from("uid")),
                                Value::String(Utf8String::from("arts80")),
                            ),
                            (
                                Value::String(Utf8String::from("rank")),
                                Value::Integer(Integer::from(100)),
                            ),
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
        super::test::write_file_str(dir.path().join("dir/group_list.yaml"), &groups);
        super::test::write_file_str(dir.path().join("dir/prefab.yaml"), &prefab);
        super::test::write_file_str(dir.path().join("room.yaml"), &room);
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
