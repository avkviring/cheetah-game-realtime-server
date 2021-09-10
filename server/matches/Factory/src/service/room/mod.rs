use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use cheetah_microservice::tonic::codegen::http::Error;

use crate::proto::matches::relay::types as relay;

use self::group::{GroupAlias, GroupResolver};
use self::prefab::PrefabResolver;

pub mod error;
mod group;
pub mod loader;
mod prefab;

pub type FieldAlias = String;
pub type PrefabAlias = PathBuf;

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
	#[serde(default)]
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
