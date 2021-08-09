mod keyvalue;

use crate::proto::matches::relay::types as relay;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    PrefabNotFound(PathBuf, String),
    GroupNotFound(PathBuf, String),
    Io(io::Error),
    Yaml(serde_yaml::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err)
    }
}

impl Error {
    pub fn stringify(self, name: &str) -> String {
        match self {
            Error::PrefabNotFound(path, name) => {
                format!("Prefab {} not found in {}", name, path.display())
            }
            Error::GroupNotFound(path, name) => {
                format!("Group {} not found in {}", name, path.display())
            }
            Error::Io(err) => format!("{:?} IO: {:?}", name, err),
            Error::Yaml(err) => format!(
                "{:?} Wrong file format {:?}: {:?}",
                name,
                err.location().map(|loc| (loc.line(), loc.column())),
                err
            ),
        }
    }
}

/// Описание комнаты
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Room {
    /// Путь до файла со всеми группами
    pub groups: PathBuf,
    /// Шаблоны для создания объектов
    pub prefabs: HashMap<String, PathBuf>,
    /// Объекты комнаты
    #[serde(with = "keyvalue")]
    pub objects: HashMap<String, Object>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Prefab {
    /// Права доступа для всего объекта
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub access: HashMap<String, Rule>,
    /// Права доступа и настройки по умолчанию для объектов
    #[serde(with = "keyvalue")]
    pub fields: HashMap<String, PrefabField>,
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
    pub id: String,
    #[serde(flatten)]
    pub field: OptionValue,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub access: HashMap<String, Rule>,
}

/// Описание объекта в комнате
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Object {
    /// Короткое имя объекта (для keyvalue)
    pub id: String,

    /// Имя префаба
    pub prefab: String,
    /// Имя группы
    pub group: String,
    /// Поля объекта
    #[serde(default, with = "keyvalue")]
    pub fields: Fields,
}

type Fields = HashMap<String, ObjectField>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ObjectField {
    /// Имя поля в объекте
    pub id: String,
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
    fn into_field(self) -> Option<FieldValue> {
        Some(match self {
            OptionValue::Struct { value: Some(value) } => FieldValue::Struct { value },
            OptionValue::I64 { value: Some(value) } => FieldValue::I64 { value },
            OptionValue::F64 { value: Some(value) } => FieldValue::F64 { value },
            OptionValue::Event => FieldValue::Event,
            _ => return None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Groups {
    #[serde(with = "keyvalue")]
    pub groups: HashMap<String, Group>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub mask: u64,
}

impl keyvalue::KeyValue<'_> for ObjectField {
    type Key = String;
    fn key(&self) -> Self::Key {
        self.id.clone()
    }
}

impl keyvalue::KeyValue<'_> for PrefabField {
    type Key = String;
    fn key(&self) -> Self::Key {
        self.id.clone()
    }
}

impl keyvalue::KeyValue<'_> for Object {
    type Key = String;
    fn key(&self) -> Self::Key {
        self.id.clone()
    }
}

impl keyvalue::KeyValue<'_> for Group {
    type Key = String;
    fn key(&self) -> Self::Key {
        self.id.clone()
    }
}

pub fn load_dir(dir: &Path) -> Result<HashMap<String, relay::RoomTemplate>, String> {
    load_room_dir(dir, Path::new("/"))
}

fn load_room_dir(
    dir: &Path,
    prefix: &Path,
) -> Result<HashMap<String, relay::RoomTemplate>, String> {
    dir.read_dir()
        .unwrap()
        .filter_map(Result::ok)
        // пропускаем служебные каталоги при монтировании ConfigMap в kubernetes
        .filter(|entry| entry.path().to_str().map_or(false, |p| !p.contains("..")))
        .try_fold(HashMap::new(), |mut result, entry| {
            let name = entry.file_name().into_string().unwrap();

            if entry.file_type().unwrap().is_dir() {
                let prefix = prefix.join(name);
                result.extend(load_room_dir(&entry.path(), &prefix)?);
            } else if let Some(name) = name
                .strip_suffix(".yaml")
                .or_else(|| name.strip_suffix(".yml"))
            {
                let name = prefix.join(name).display().to_string();
                if let Some(room) =
                    load_room(entry.path(), dir).map_err(|err| err.stringify(&name))?
                {
                    result.insert(name, room);
                }
            }

            Ok(result)
        })
}

fn load_room(path: impl AsRef<Path>, dir: &Path) -> Result<Option<relay::RoomTemplate>, Error> {
    let path = &dir.join(path);

    let file = std::fs::File::open(path)?;
    let room: Room = match serde_yaml::from_reader(file) {
        Ok(room) => room,
        Err(_) => return Ok(None),
    };

    let groups = GroupResolver::load(&dir.join(room.groups))?;

    let mut prefabs = HashMap::new();
    for (id, (name, path)) in room.prefabs.into_iter().enumerate() {
        let path = dir.join(path);
        let prefab = PrefabResolver::load(id as u32, &groups, &path)?;
        prefabs.insert(name, prefab);
    }

    let templates: HashMap<&str, u32> = prefabs
        .iter()
        .map(|(name, prefab)| (name.as_str(), prefab.template.template))
        .collect();

    let objects = room
        .objects
        .into_iter()
        .enumerate()
        .map(|(id, (_, object))| -> Result<_, Error> {
            let prefab = prefabs
                .get(object.prefab.as_str())
                .ok_or_else(|| Error::PrefabNotFound(path.into(), object.prefab.clone()))?;

            let groups = groups
                .resolve_mask(&object.group)
                .ok_or_else(|| Error::GroupNotFound(path.into(), object.group.clone()))?;

            Ok(relay::GameObjectTemplate {
                id: id as u32,
                template: templates[object.prefab.as_str()],
                groups,
                fields: Some(prefab.resolve(&object.fields)),
            })
        })
        .collect::<Result<_, Error>>()?;

    let permissions = prefabs
        .values()
        .map(|prefab| prefab.template.clone())
        .collect();

    Ok(Some(relay::RoomTemplate {
        objects,
        permissions: Some(relay::Permissions {
            objects: permissions,
        }),
    }))
}

struct GroupResolver {
    groups: HashMap<String, (u32, u64)>,
}

impl GroupResolver {
    fn load(path: &Path) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let Groups { groups } = serde_yaml::from_reader(file)?;

        let groups = groups
            .into_iter()
            .enumerate()
            .map(|(id, (name, group))| (name, (id as u32, group.mask)))
            .collect();

        Ok(Self { groups })
    }

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
    fn load(template: u32, groups: &GroupResolver, path: &Path) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let prefab: Prefab = serde_yaml::from_reader(file)?;

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
                    .into_field()
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
            template,
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
            FieldValue::Struct { value } => Some((key, rmp_serde::to_vec(&value).unwrap())),
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
    let mut groups = Groups::default();

    groups.groups.insert(
        test_group.clone(),
        Group {
            id: test_group.clone(),
            mask: 1234,
        },
    );

    println!("{}", serde_yaml::to_string(&groups).unwrap());

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
        access: access.clone(),
        fields,
    };

    println!("{}", serde_yaml::to_string(&prefab).unwrap());

    let mut fields = HashMap::default();

    fields.insert(
        "test".into(),
        ObjectField {
            id: "test".into(),
            value: FieldValue::I64 { value: 12345 },
        },
    );

    let mut room = Room {
        groups: "groups.yaml".into(),
        ..Room::default()
    };
    room.prefabs.insert("foo".into(), "prefab/foo.yaml".into());
    room.prefabs.insert("bar".into(), "prefab/bar.yaml".into());
    room.objects.insert(
        "xyz".into(),
        Object {
            id: "xyz".into(),
            prefab: "foo".into(),
            group: "test_group".into(),
            fields,
        },
    );

    println!("{}", serde_yaml::to_string(&room).unwrap());
}
