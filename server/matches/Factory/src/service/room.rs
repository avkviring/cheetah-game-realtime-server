use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
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
    fn stringify(self, name: &str) -> String {
        match self {
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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Prefab {
    pub templates: HashMap<u32, PathBuf>,
    pub objects: HashMap<u32, GameObject>,
}

/// Структуры для чтения шаблона комнаты из yaml файла
/// Специально разделен с grpc типами, так как yaml представление может иметь другую структуру
/// для большего удобства
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Room {
    #[serde(default)]
    pub templates: HashMap<u32, Permissions>,
    #[serde(default)]
    pub objects: HashMap<u32, GameObject>,
}

impl Room {
    pub fn load_prefab(prefab: impl AsRef<Path>, base: impl AsRef<Path>) -> Result<Self, Error> {
        let file = std::fs::File::open(prefab)?;
        let Prefab { templates, objects } = serde_yaml::from_reader(file)?;

        let base = base.as_ref();
        let templates = templates
            .iter()
            .map(|(&key, path)| {
                let file = std::fs::File::open(base.join(path))?;
                let template = serde_yaml::from_reader(file)?;
                Ok((key, template))
            })
            .collect::<Result<_, _>>();

        templates.map(|templates| Self { templates, objects })
    }

    pub fn load_dir<T: From<Self>>(dir: &Path) -> Result<HashMap<String, T>, String> {
        Self::load(dir, Path::new("/"))
    }

    fn load<T: From<Self>>(dir: &Path, prefix: &Path) -> Result<HashMap<String, T>, String> {
        dir.read_dir()
            .unwrap()
            .filter_map(Result::ok)
            // пропускаем служебные каталоги при монтировании ConfigMap в kubernetes
            .filter(|entry| entry.path().to_str().map_or(false, |p| !p.contains("..")))
            .try_fold(HashMap::new(), |mut result, entry| {
                let name = entry.file_name().into_string().unwrap();

                if entry.file_type().unwrap().is_dir() {
                    let prefix = prefix.join(name);
                    let rooms = Self::load(&entry.path(), &prefix)?;
                    result.extend(rooms);
                } else if let Some(name) = name
                    .strip_suffix(".yaml")
                    .or_else(|| name.strip_suffix(".yml"))
                {
                    let name = prefix.join(name).display().to_string();

                    let room =
                        Self::load_prefab(entry.path(), dir).map_err(|err| err.stringify(&name))?;

                    result.insert(name, room.into());
                }

                Ok(result)
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct GameObject {
    pub template: u32,
    pub groups: u64,

    #[serde(default)]
    pub fields: HashMap<u32, ObjectField>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ObjectField {
    I64 { value: i64 },
    F64 { value: f64 },
    Struct { value: rmpv::Value },
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Permissions {
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub fields: HashMap<u32, PermissionField>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "permission")]
pub enum Rule {
    #[serde(rename = "deny")]
    Deny { groups: u64 },
    #[serde(rename = "ro")]
    ReadOnly { groups: u64 },
    #[serde(rename = "rw")]
    ReadWrite { groups: u64 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PermissionField {
    I64 { rules: Vec<Rule> },
    F64 { rules: Vec<Rule> },
    Struct { rules: Vec<Rule> },
    Event { rules: Vec<Rule> },
}

#[cfg(test)]
mod tests {
    use crate::service::room::{
        GameObject, ObjectField, PermissionField, Permissions, Prefab, Room, Rule,
    };
    use crate::service::test::write_file;
    use rmpv::{Integer, Utf8String, Value};
    use std::collections::HashMap;

    #[test]
    pub fn should_load_prefab() {
        let tmp = tempfile::TempDir::new().unwrap();
        let prefab = Prefab {
            templates: std::iter::once((1, "templates/perm.yaml".into())).collect(),
            objects: HashMap::default(),
        };
        let prefab_data = serde_yaml::to_string(&prefab).unwrap();

        let perm = Permissions {
            rules: vec![Rule::Deny { groups: 12495 }],
            fields: std::iter::once((
                100,
                PermissionField::I64 {
                    rules: vec![Rule::ReadOnly { groups: 5677 }],
                },
            ))
            .collect(),
        };

        let perm = serde_yaml::to_string(&perm).unwrap();

        write_file(tmp.path().join("prefab.yaml"), &prefab_data).unwrap();
        write_file(tmp.path().join("templates/perm.yaml"), &perm).unwrap();

        let room = Room::load_prefab(tmp.path().join("prefab.yaml"), tmp).unwrap();

        dbg!(serde_yaml::to_string(&room).unwrap());
    }

    #[test]
    #[ignore]
    fn dump_yaml_for_docs() {
        let room = Room {
            objects: std::iter::once((
                555,
                GameObject {
                    template: 1,
                    groups: 4857,
                    fields: IntoIterator::into_iter([
                        (10, ObjectField::I64 { value: 100100 }),
                        (20, ObjectField::F64 { value: 3.14 }),
                        (30, ObjectField::F64 { value: 2.68 }),
                        (
                            40,
                            ObjectField::Struct {
                                value: Value::Map(vec![
                                    (
                                        Value::String(Utf8String::from("uid")),
                                        Value::String(Utf8String::from("arts80")),
                                    ),
                                    (
                                        Value::String(Utf8String::from("rank")),
                                        Value::Integer(Integer::from(100)),
                                    ),
                                ]),
                            },
                        ),
                        (
                            50,
                            ObjectField::Struct {
                                value: Value::Array(vec![
                                    Value::Integer(Integer::from(15)),
                                    Value::Integer(Integer::from(26)),
                                ]),
                            },
                        ),
                    ])
                    .collect(),
                },
            ))
            .collect(),
            templates: std::iter::once((
                1,
                Permissions {
                    rules: vec![Rule::Deny { groups: 12495 }],
                    fields: std::iter::once((
                        100,
                        PermissionField::I64 {
                            rules: vec![Rule::ReadOnly { groups: 5677 }],
                        },
                    ))
                    .collect(),
                },
            ))
            .collect(),
        };

        panic!("{}", serde_yaml::to_string(&room).unwrap());
    }
}
