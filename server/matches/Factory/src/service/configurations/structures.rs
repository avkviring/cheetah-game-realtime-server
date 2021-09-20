use std::collections::HashMap;

use serde::Deserialize;

pub type FieldName = String;
pub type TemplateName = String;
pub type RoomName = String;
pub type GroupName = String;

/// Описание комнаты
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Room {
	/// Объекты комнаты
	pub objects: Vec<RoomObject>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Field {
	pub id: u16,
	pub r#type: FieldType,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Template {
	pub id: u32,
	#[serde(default)]
	pub permissions: TemplatePermissions,
}

#[derive(Debug, Deserialize, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplatePermissions {
	/// Права доступа для всего объекта
	#[serde(default)]
	pub groups: HashMap<GroupName, PermissionLevel>,
	/// Права доступа и настройки по умолчанию для объектов
	#[serde(default)]
	pub fields: Vec<PermissionField>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PermissionField {
	pub field: FieldName,
	#[serde(default)]
	pub groups: HashMap<GroupName, PermissionLevel>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
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
	#[serde(default)]
	pub id: u32,
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
