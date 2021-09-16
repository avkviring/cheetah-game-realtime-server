use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::service::configurations::error::Error;
use crate::service::configurations::structures::{Field, FieldName, GroupName, Room, RoomName, Template, TemplateName};

pub mod error;
pub mod structures;

///
/// Загруженная информация из каталога с конфигурацией
///
#[derive(Default, Debug)]
pub struct Configurations {
	pub groups: HashMap<GroupName, u64>,
	pub fields: HashMap<FieldName, Field>,
	pub templates: HashMap<TemplateName, Template>,
	pub rooms: HashMap<RoomName, Room>,
}

impl Configurations {
	pub fn load(root: impl Into<PathBuf>) -> Result<Self, Error> {
		let root = root.into();
		let groups = Self::load_group(root.clone())?;
		let fields = Self::load_items::<_>(root.join("fields").as_path(), Path::new("/"))?;
		let templates = Self::load_items::<_>(root.join("templates").as_path(), Path::new("/"))?;
		let rooms = Self::load_items::<_>(root.join("rooms").as_path(), Path::new("/"))?;
		Result::Ok(Configurations {
			groups,
			fields,
			templates,
			rooms,
		})
	}

	fn load_group(root: PathBuf) -> Result<HashMap<GroupName, u64>, Error> {
		let yaml = root.join("groups.yaml");
		let group_file = if yaml.exists() { yaml } else { root.join("groups.yml") };
		let content = read_to_string(group_file).map_err(|_| Error::GroupFileNotFound)?;
		serde_yaml::from_str::<_>(content.as_ref()).map_err(Error::GroupParseError)
	}

	fn load_items<T>(dir: &Path, prefix: &Path) -> Result<HashMap<String, T>, Error>
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
				let sub_entities = Self::load_items(&entry.path(), &prefix)?;
				sub_entities.into_iter().for_each(|(k, v)| {
					result.insert(k, v);
				});
			} else if let Some(name) = name.strip_suffix(".yaml").or_else(|| name.strip_suffix(".yml")) {
				let name = prefix.join(name);
				let content = read_to_string(entry.path())?;
				let value = serde_yaml::from_str(content.as_ref()).map_err(Error::Yaml)?;
				result.insert(name.to_str().unwrap().to_string(), value);
			}
		}

		Ok(result)
	}
}
