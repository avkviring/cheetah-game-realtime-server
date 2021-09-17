use std::collections::HashMap;

use serde::Deserialize;

pub type FieldName = String;
pub type TemplateName = String;
pub type RoomName = String;
pub type GroupName = String;

/// Описание комнаты
#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Room {
	/// Объекты комнаты
	pub objects: Vec<RoomObject>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Field {
	pub id: u16,
	pub r#type: FieldType,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Template {
	pub id: u32,
	#[serde(default)]
	pub permissions: TemplatePermissions,
}

#[derive(Debug, Deserialize, Clone, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TemplatePermissions {
	/// Права доступа для всего объекта
	#[serde(default)]
	pub groups: HashMap<GroupName, Permission>,
	/// Права доступа и настройки по умолчанию для объектов
	#[serde(default)]
	pub fields: Vec<PermissionField>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PermissionField {
	pub field: FieldName,
	#[serde(default)]
	pub groups: HashMap<GroupName, Permission>,
}

#[derive(Debug, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum Permission {
	#[serde(rename = "deny")]
	Deny,
	#[serde(rename = "ro")]
	ReadOnly,
	#[serde(rename = "rw")]
	ReadWrite,
}

/// Описание объекта в комнате
#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Debug, Deserialize, Clone, PartialEq)]
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
