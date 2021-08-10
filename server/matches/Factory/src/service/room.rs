mod keyvalue;

use crate::proto::matches::relay::types as relay;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub type GroupAlias = String;
pub type FieldAlias = String;
pub type PrefabAlias = PathBuf;

#[derive(Debug)]
pub enum Error {
    PrefabNotFound(PathBuf, PrefabAlias),
    GroupNotFound(PathBuf, String),
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
#[serde(deny_unknown_fields)]
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
    #[serde(skip_serializing_if = "skip_path")]
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
    #[serde(skip_serializing_if = "skip_path")]
    pub groups: PathBuf,
    /// Права доступа для всего объекта
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub access: HashMap<GroupAlias, Rule>,
    /// Права доступа и настройки по умолчанию для объектов
    #[serde(with = "keyvalue")]
    pub fields: HashMap<FieldAlias, PrefabField>,
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
#[serde(deny_unknown_fields)]
pub struct PrefabField {
    pub id: FieldAlias,
    #[serde(flatten)]
    pub field: OptionValue,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
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
    #[serde(default, with = "keyvalue", skip_serializing_if = "HashMap::is_empty")]
    pub fields: Fields,
}

type Fields = HashMap<FieldAlias, ObjectField>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ObjectField {
    /// Имя поля в объекте
    pub field: FieldAlias,
    /// Значение поля (может быть загружено из префаба)
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

impl keyvalue::KeyValue<'_> for ObjectField {
    type Key = FieldAlias;
    fn key(&self) -> Self::Key {
        self.field.clone()
    }
}

impl keyvalue::KeyValue<'_> for PrefabField {
    type Key = FieldAlias;
    fn key(&self) -> Self::Key {
        self.id.clone()
    }
}

#[derive(Default)]
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
        let (local_groups, global_groups) = {
            let mut last_id = 1_u32;
            let mut global = HashMap::new();
            let local: HashMap<_, _> = groups
                .into_iter()
                .map(|(path, raw)| {
                    let mut groups = HashMap::default();
                    for (name, mask) in raw {
                        let id = last_id;
                        last_id += 1;
                        groups.insert(name, (id, mask));
                    }
                    global.extend(groups.clone());
                    (path, GroupResolver { groups })
                })
                .collect();
            (local, GroupResolver { groups: global })
        };

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
                            template: prefab.template.template,
                            groups,
                            fields: Some(prefab.resolve(&object.fields)),
                        })
                    })
                    .collect::<Result<_, Error>>()?;

                let permissions = all_prefabs
                    .values()
                    .map(|prefab| prefab.template.clone())
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
#[derive(Default)]
struct GroupResolver {
    groups: HashMap<String, (u32, u64)>,
}

impl GroupResolver {
    fn resolve_mask(&self, group: &str) -> Option<u64> {
        self.groups.get(group).map(|(_, mask)| *mask)
    }

    fn resolve(
        &self,
        group: &str,
        rule: Rule,
        path: &Path,
    ) -> Result<relay::GroupsPermissionRule, Error> {
        let permission = match rule {
            Rule::Deny => relay::PermissionLevel::Deny as i32,
            Rule::ReadOnly => relay::PermissionLevel::Ro as i32,
            Rule::ReadWrite => relay::PermissionLevel::Rw as i32,
        };

        self.groups
            .get(group)
            .copied()
            .map(|(_, groups)| relay::GroupsPermissionRule { groups, permission })
            .ok_or_else(|| Error::GroupNotFound(path.into(), group.into()))
    }
}

struct PrefabResolver {
    template: relay::GameObjectTemplatePermission,
    defaults: HashMap<u32, FieldValue>,
    field_names: HashMap<String, u32>,
}

impl PrefabResolver {
    fn new(prefab: Prefab, groups: &GroupResolver, path: &Path) -> Result<Self, Error> {
        let rules = prefab
            .access
            .into_iter()
            .map(|(name, rule)| groups.resolve(&name, rule, path))
            .collect::<Result<_, Error>>()?;

        let field_names: HashMap<String, u32> = prefab
            .fields
            .keys()
            .enumerate()
            .map(|(id, name)| (name.clone(), id as u32))
            .collect();

        let defaults = prefab
            .fields
            .iter()
            .filter_map(|(name, field)| {
                field
                    .field
                    .clone()
                    .into_value()
                    .map(|field| (field_names[name], field))
            })
            .collect();

        let fields = prefab
            .fields
            .into_iter()
            .map(|(name, field)| {
                let id = field_names[&name];

                let r#type = match field.field {
                    OptionValue::I64 { .. } => relay::FieldType::Long as i32,
                    OptionValue::F64 { .. } => relay::FieldType::Float as i32,
                    OptionValue::Struct { .. } => relay::FieldType::Structure as i32,
                    OptionValue::Event { .. } => relay::FieldType::Event as i32,
                };

                let rules = field
                    .access
                    .into_iter()
                    .map(|(name, rule)| groups.resolve(&name, rule, path))
                    .collect::<Result<_, Error>>()?;

                Ok(relay::PermissionField { id, r#type, rules })
            })
            .collect::<Result<_, Error>>()?;

        let template = relay::GameObjectTemplatePermission {
            template: prefab.template,
            rules,
            fields,
        };

        Ok(Self {
            template,
            defaults,
            field_names,
        })
    }

    fn resolve(&self, object: &HashMap<String, ObjectField>) -> relay::GameObjectFieldsTemplate {
        let mut fields = self.defaults.clone();
        let mut last_id = fields.keys().copied().max().unwrap_or_default();

        for (name, field) in object {
            let id = self.field_names.get(name).copied().unwrap_or_else(|| {
                last_id += 1;
                last_id as u32
            });
            fields.insert(id, field.value.clone());
        }

        let longs = fields.iter().filter_map(|(&key, field)| match field {
            FieldValue::I64 { value } => Some((key, *value)),
            _ => None,
        });
        let floats = fields.iter().filter_map(|(&key, field)| match field {
            FieldValue::F64 { value } => Some((key, *value)),
            _ => None,
        });
        let structures = fields.iter().filter_map(|(&key, field)| match field {
            FieldValue::Struct { value } => Some((key, rmp_serde::to_vec(&value).ok()?)),
            _ => None,
        });

        relay::GameObjectFieldsTemplate {
            longs: longs.collect(),
            floats: floats.collect(),
            structures: structures.collect(),
        }
    }
}

#[cfg(test)]
#[test]
fn doc() {
    use rmpv::{Integer, Utf8String, Value};

    let test_group = "test_group".to_string();
    {
        let mut groups = HashMap::default();
        groups.insert(test_group.clone(), 1234);

        let config = Config::Groups { groups };
        println!("{}", serde_yaml::to_string(&config).unwrap());
    }

    {
        let mut fields = HashMap::new();

        let mut access = HashMap::default();
        access.insert(test_group, Rule::Deny);

        let value = Value::Map(vec![
            (
                Value::String(Utf8String::from("uid")),
                Value::String(Utf8String::from("arts80")),
            ),
            (
                Value::String(Utf8String::from("rank")),
                Value::Integer(Integer::from(100)),
            ),
        ]);

        fields.insert(
            "a".to_string(),
            PrefabField {
                id: "a".to_string(),
                access: access.clone(),
                field: OptionValue::Struct { value: Some(value) },
            },
        );

        fields.insert(
            "b".to_string(),
            PrefabField {
                id: "b".to_string(),
                access: access.clone(),
                field: OptionValue::I64 { value: Some(4) },
            },
        );

        fields.insert(
            "c".to_string(),
            PrefabField {
                id: "c".to_string(),
                access: HashMap::default(),
                field: OptionValue::F64 { value: None },
            },
        );

        let prefab = Prefab {
            template: 1234,
            groups: PathBuf::new(),
            access: access.clone(),
            fields,
        };

        let config = Config::Prefab(prefab);
        println!("{}", serde_yaml::to_string(&config).unwrap());
    }

    {
        let mut fields = HashMap::default();

        fields.insert(
            "test".into(),
            ObjectField {
                field: "test".into(),
                value: FieldValue::I64 { value: 12345 },
            },
        );

        let mut room = Room {
            groups: "/dir/some_group_list".into(),
            ..Room::default()
        };
        room.prefabs.insert("foo".into(), "/prefab/foo".into());
        room.prefabs.insert("bar".into(), "/prefab/bar".into());
        room.objects.push(Object {
            prefab: "/dir/prefab".into(),
            group: "test_group".into(),
            fields: Default::default(),
        });
        room.objects.push(Object {
            prefab: "foo".into(),
            group: "test_group".into(),
            fields,
        });

        let config = Config::Room(room);
        println!("{}", serde_yaml::to_string(&config).unwrap());
    }
}
