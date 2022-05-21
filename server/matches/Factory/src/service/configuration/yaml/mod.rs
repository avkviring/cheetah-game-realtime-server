///
/// Исходная конфигурация в yaml формате
///
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::service::configuration::yaml::error::Error;
use crate::service::configuration::yaml::structures::{
	Field, FieldName, FieldType, GroupName, Room, RoomName, SelfName, Template, TemplateName,
};

pub mod error;
pub mod structures;

///
/// Загруженная информация из каталога с конфигурацией
///
#[derive(Default, Debug)]
pub struct YamlConfigurations {
	pub groups: HashMap<GroupName, u64>,
	pub fields: HashMap<FieldName, Field>,
	pub templates: HashMap<TemplateName, Template>,
	pub rooms: HashMap<RoomName, Room>,
}

impl YamlConfigurations {
	pub fn load(root: impl Into<PathBuf>) -> Result<Self, Error> {
		let root = root.into();
		let groups = Self::load_group(root.clone())?;
		let fields = Self::load_items::<_>(
			root.clone(), root.join("fields").as_path(),
			Path::new(""), || None
		)?;
		let templates = Self::load_items::<_>(
			root.clone(), root.join("templates").as_path(),
			Path::new(""), || None
		)?;
		let rooms = Self::load_items::<_>(
			root.clone(), root.join("rooms").as_path(), Path::new(""), 
			|| {
				Some(Room { objects: vec![] })
			}
		)?;
		YamlConfigurations {
			groups,
			fields,
			templates,
			rooms,
		}
		.validate()
	}

	fn validate(self) -> Result<Self, Error> {
		self.validate_templates()?.validate_fields()
	}

	fn validate_fields(self) -> Result<YamlConfigurations, Error> {
		let mut exist_fields: HashMap<(FieldType, u16), String> = HashMap::default();
		for (name, field) in self.fields.iter() {
			let key = (field.r#type.clone(), field.id);
			if let Vacant(e) = exist_fields.entry(key.clone()) {
				e.insert(name.clone());
			} else {
				return Err(Error::FieldWithSameIdAlreadyExists {
					id: field.id,
					exist: exist_fields.get(&key).unwrap().clone(),
					current: name.clone(),
				});
			}
		}
		Result::Ok(self)
	}

	fn validate_templates(self) -> Result<YamlConfigurations, Error> {
		let mut exist_templates: HashMap<u32, String> = HashMap::default();
		for (name, template) in self.templates.iter() {
			if let Vacant(e) = exist_templates.entry(template.id) {
				e.insert(name.clone());
			} else {
				return Err(Error::TemplateWithSameIdAlreadyExists {
					id: template.id,
					exist: exist_templates.get(&template.id).unwrap().clone(),
					current: name.clone(),
				});
			}
		}
		Result::Ok(self)
	}

	fn load_group(root: PathBuf) -> Result<HashMap<GroupName, u64>, Error> {
		let yaml = root.join("groups.yaml");
		let group_file = if yaml.exists() { yaml } else { root.join("groups.yml") };
		let content = read_to_string(group_file.clone()).map_err(|_| Error::GroupFileNotFound)?;
		serde_yaml::from_str::<_>(content.as_ref()).map_err(|e| Error::Yaml {
			global_root: root.clone(),
			file: group_file.clone(),
			e,
		})
	}

	///
	/// Default это временный костыль, так как serde_yaml не может распарсить пустую строку в
	/// структуру со всеми полями по-умолчанию
	///
	/// Убрать после закрытия задачи - https://github.com/dtolnay/serde-yaml/issues/86
	///
	fn load_items<T: SelfName>(
		global_root: PathBuf,
		dir: &Path,
		prefix: &Path,
		default_factory: fn() -> Option<T>,
	) -> Result<HashMap<String, T>, Error>
	where
		T: DeserializeOwned,
	{
		let mut result = HashMap::default();
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
				let sub_entities = Self::load_items(global_root.clone(), &entry.path(), &prefix, default_factory)?;
				sub_entities.into_iter().for_each(|(k, v)| {
					result.insert(k, v);
				});
			} else if let Some(name) = name.strip_suffix(".yaml").or_else(|| name.strip_suffix(".yml")) {
				let name = prefix.join(name);
				let path = entry.path();
				let content = read_to_string(&path)?;

				let mut count = 0;
				let name_from_path = name.to_str().unwrap().to_string().replace('\\', "/");
				let prepared_content = YamlConfigurations::prepare_content(content);
				for document in serde_yaml::Deserializer::from_str(prepared_content.as_str()) {
					count += 1;
					let value = T::deserialize(document).map_err(|e| Error::Yaml {
						global_root: global_root.clone(),
						file: path.clone(),
						e,
					})?;
					let name_from_item = value
						.get_self_name()
						.map(|v| format!("/{}", v))
						.unwrap_or_else(|| "".to_string());

					let key = format!("{}{}", name_from_path, name_from_item);
					match result.insert(key.clone(), value) {
						None => {}
						Some(_) => return Err(Error::NameAlreadyExists { name: key, file: path }),
					}
				}
				// файл пустой - необходимо создать структуру по-умолчанию
				if count == 0 {
					match default_factory() {
						None => {
							return Err(Error::CannotCreateDefault {
								name: name_from_path,
								file: path,
							})
						}
						Some(value) => match result.insert(name_from_path.clone(), value) {
							None => {}
							Some(_) => {
								return Err(Error::NameAlreadyExists {
									name: name_from_path,
									file: path,
								})
							}
						},
					}
				}
			}
		}

		Ok(result)
	}

	fn prepare_content(content: String) -> String {
		content.replace("\u{feff}", "")
	}
}

#[cfg(test)]
pub mod test {
	use include_dir::{include_dir, Dir};
	use rmpv::{Integer, Utf8String};

	use crate::service::configuration::yaml::error::Error;
	use crate::service::configuration::yaml::structures::{
		Field, FieldType, FieldValue, PermissionField, PermissionLevel, Room, RoomObject, Template, TemplatePermissions,
	};
	use crate::service::configuration::yaml::YamlConfigurations;

	pub const EXAMPLE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/example/rooms/");

	#[test]
	pub fn should_load_groups() {
		let configuration = setup();
		assert_eq!(
			configuration.groups,
			vec![
				("red".to_string(), 1),
				("blue".to_string(), 2),
				("bot".to_string(), 4),
				("user".to_string(), 8)
			]
			.into_iter()
			.collect()
		);
	}

	#[test]
	pub fn should_load_fields() {
		let configuration = setup();
		assert_eq!(
			configuration.fields,
			vec![
				(
					"characteristic/damage".to_string(),
					Field {
						name: None,
						id: 10,
						r#type: FieldType::Double
					}
				),
				(
					"characteristic/healing".to_string(),
					Field {
						name: None,
						id: 15,
						r#type: FieldType::Double
					}
				),
				(
					"user/info".to_string(),
					Field {
						name: None,
						id: 1,
						r#type: FieldType::Struct
					}
				),
				(
					"user/score".to_string(),
					Field {
						name: None,
						id: 2,
						r#type: FieldType::Long
					}
				),
				(
					"multi/power".to_string(),
					Field {
						name: Some("power".to_string()),
						id: 100,
						r#type: FieldType::Double
					}
				),
				(
					"multi/info".to_string(),
					Field {
						name: Some("info".to_string()),
						id: 110,
						r#type: FieldType::Struct
					}
				),
				(
					"with_bom".to_string(),
					Field {
						name: None,
						id: 55,
						r#type: FieldType::Struct
					}
				)
			]
			.into_iter()
			.collect()
		);
	}

	#[test]
	pub fn should_load_templates() {
		let configuration = setup();
		assert_eq!(
			configuration.templates,
			vec![
				(
					"weapons/turret".to_string(),
					Template {
						id: 100,
						permissions: TemplatePermissions {
							groups: Default::default(),
							fields: vec![PermissionField {
								field: "characteristic/damage".to_string(),
								groups: vec![("bot".to_string(), PermissionLevel::Deny)].into_iter().collect()
							}]
						}
					}
				),
				(
					"user".to_string(),
					Template {
						id: 1,
						permissions: TemplatePermissions {
							groups: vec![
								("bot".to_string(), PermissionLevel::Deny),
								("user".to_string(), PermissionLevel::ReadWrite),
							]
							.into_iter()
							.collect(),
							fields: vec![PermissionField {
								field: "user/score".to_string(),
								groups: vec![("bot".to_string(), PermissionLevel::ReadWrite)].into_iter().collect(),
							}]
						}
					}
				)
			]
			.into_iter()
			.collect()
		);
	}

	#[test]
	pub fn should_load_rooms() {
		let configuration = setup();
		assert_eq!(
			configuration.rooms,
			vec![
				(
					"gubaha".to_string(),
					Room {
						objects: vec![
							RoomObject {
								id: Some(100),
								template: "user".to_string(),
								group: "red".to_string(),
								values: vec![
									FieldValue {
										field: "user/score".to_string(),
										value: rmpv::Value::Integer(Integer::from(100))
									},
									FieldValue {
										field: "user/info".to_string(),
										value: rmpv::Value::Map(vec![(
											rmpv::Value::String(Utf8String::from("name")),
											rmpv::Value::String(Utf8String::from("alex"))
										)])
									}
								]
							},
							RoomObject {
								id: None,
								template: "weapons/turret".to_string(),
								group: "blue".to_string(),
								values: vec![FieldValue {
									field: "characteristic/damage".to_string(),
									value: rmpv::Value::Integer(Integer::from(200))
								}]
							}
						]
					}
				),
				("kungur".to_string(), Room { objects: vec![] })
			]
			.into_iter()
			.collect()
		)
	}

	#[test]
	pub fn validate_unique_template_id() {
		let configurations = YamlConfigurations {
			groups: Default::default(),
			fields: Default::default(),
			templates: vec![
				(
					"templateA".to_string(),
					Template {
						id: 100,
						permissions: Default::default(),
					},
				),
				(
					"templateB".to_string(),
					Template {
						id: 100,
						permissions: Default::default(),
					},
				),
			]
			.into_iter()
			.collect(),
			rooms: Default::default(),
		};
		let result = configurations.validate();
		assert!(matches!(
			result,
			Result::Err(Error::TemplateWithSameIdAlreadyExists {
				id,
				exist,
				current
			}) if id==100
			&& exist!=current
			&& ["templateA", "templateB"].contains(&exist.as_str())
			&& ["templateA", "templateB"].contains(&current.as_str())
		));
	}

	#[test]
	pub fn validate_unique_field_id() {
		let configurations = YamlConfigurations {
			groups: Default::default(),
			fields: vec![
				(
					"fieldA".to_string(),
					Field {
						name: None,
						id: 100,
						r#type: FieldType::Long,
					},
				),
				(
					"fieldB".to_string(),
					Field {
						name: None,
						id: 100,
						r#type: FieldType::Long,
					},
				),
			]
			.into_iter()
			.collect(),
			templates: Default::default(),
			rooms: Default::default(),
		};
		let result = configurations.validate();
		assert!(matches!(
			result,
			Result::Err(Error::FieldWithSameIdAlreadyExists {
				id,
				exist,
				current
			}) if id==100
			&& exist!=current
			&& ["fieldA", "fieldB"].contains(&exist.as_str())
			&& ["fieldA", "fieldB"].contains(&current.as_str())
		));
	}

	#[test]
	pub fn validate_unique_field_id_for_different_types() {
		let configurations = YamlConfigurations {
			groups: Default::default(),
			fields: vec![
				(
					"fieldA".to_string(),
					Field {
						name: None,
						id: 100,
						r#type: FieldType::Long,
					},
				),
				(
					"fieldB".to_string(),
					Field {
						name: None,
						id: 100,
						r#type: FieldType::Struct,
					},
				),
			]
			.into_iter()
			.collect(),
			templates: Default::default(),
			rooms: Default::default(),
		};

		assert!(configurations.validate().is_ok())
	}

	fn setup() -> YamlConfigurations {
		let temp_dir = tempfile::tempdir().unwrap();
		let path = temp_dir.into_path();
		EXAMPLE_DIR.extract(path.clone()).unwrap();
		YamlConfigurations::load(path).unwrap()
	}
}
