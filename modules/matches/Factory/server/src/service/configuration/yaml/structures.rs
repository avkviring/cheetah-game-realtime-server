use std::collections::HashMap;

use serde::Deserialize;

pub type FieldName = String;
pub type TemplateName = String;
pub type RoomName = String;
pub type GroupName = String;

///
/// Если задано - то полное имя определяется из пути до файла + собственное имя, если не задано -
/// то имя определяется только по пути
///
pub trait MaybeNamed {
	fn name(&self) -> Option<&str>;
}

/// Описание комнаты
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Room {
	#[serde(default)]
	pub objects: Vec<RoomObject>,
}

impl MaybeNamed for Room {
	fn name(&self) -> Option<&str> {
		None
	}
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Field {
	/// Имя опционально, актуально только для мультидокументого файла
	pub name: Option<String>,
	pub id: u16,
	#[serde(rename = "type")]
	pub typ: FieldType,
}

impl MaybeNamed for Field {
	fn name(&self) -> Option<&str> {
		self.name.as_deref()
	}
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Template {
	pub id: u32,
	#[serde(default)]
	pub permissions: TemplatePermissions,
}

impl MaybeNamed for Template {
	fn name(&self) -> Option<&str> {
		None
	}
}

#[derive(Debug, Deserialize, Default, Eq, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct TemplatePermissions {
	/// Права доступа для всего объекта
	#[serde(default)]
	pub groups: HashMap<GroupName, PermissionLevel>,
	/// Права доступа и настройки по умолчанию для объектов
	#[serde(default)]
	pub fields: Vec<PermissionField>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct PermissionField {
	pub field: FieldName,
	#[serde(default)]
	pub groups: HashMap<GroupName, PermissionLevel>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub enum PermissionLevel {
	#[serde(rename = "deny")]
	Deny,
	#[serde(rename = "ro")]
	ReadOnly,
	#[serde(rename = "rw")]
	ReadWrite,
}

/// Описание объекта в комнате
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoomObject {
	/// опциональный идентификатор объекта
	/// если не задан - то используется генерация идентификаторов
	pub id: Option<u32>,
	/// Имя шаблона
	pub template: TemplateName,
	/// Имя группы
	pub group: GroupName,
	/// Поля объекта
	#[serde(default)]
	pub values: Vec<FieldValue>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FieldValue {
	pub field: FieldName,
	pub value: rmpv::Value,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum FieldType {
	Long,
	Double,
	Struct,
	Event,
}
