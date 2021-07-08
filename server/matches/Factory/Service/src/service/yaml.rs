use std::collections::HashMap;

use serde::{Deserialize, Serialize};

///
/// Шаблон для матча
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RoomTemplate {
    pub id: u64,
    #[serde(default)]
    pub objects: Vec<GameObjectTemplate>,
    #[serde(default)]
    pub permissions: Vec<TemplatePermission>,
    #[serde(flatten)]
    pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectTemplate {
    pub id: u32,
    pub template: u16,
    pub access_groups: u64,
    #[serde(default)]
    pub fields: FieldsTemplate,
    #[serde(flatten)]
    pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FieldsTemplate {
    #[serde(default)]
    pub longs: HashMap<u16, i64>,
    #[serde(default)]
    pub floats: HashMap<u16, f64>,
    #[serde(default)]
    pub structures: HashMap<u16, rmpv::Value>,
    #[serde(flatten)]
    pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TemplatePermission {
    pub template: u16,
    #[serde(default)]
    pub groups: Vec<PermissionGroup>,
    #[serde(default)]
    pub fields: Vec<PermissionField>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct PermissionGroup {
    pub group: u64,
    pub permission: Permission,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionField {
    pub field_id: u16,
    pub field_type: FieldType,
    #[serde(default)]
    pub groups: Vec<PermissionGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum Permission {
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "ro")]
    Ro,
    #[serde(rename = "rw")]
    Rw,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum FieldType {
    #[serde(rename = "long")]
    Long,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "structure")]
    Structure,
    #[serde(rename = "event")]
    Event,
}

#[derive(Debug)]
pub enum RoomTemplateError {
    YamlParserError(serde_yaml::Error),
    YamlContainsUnmappingFields(Vec<String>),
}

impl RoomTemplate {
    const CLIENT_OBJECT_ID_OFFSET: u32 = 512;

    pub fn new_from_yaml(content: &str) -> Result<RoomTemplate, RoomTemplateError> {
        let template = serde_yaml::from_str::<RoomTemplate>(content);
        match template {
            Ok(template) => template.validate(),
            Err(e) => Result::Err(RoomTemplateError::YamlParserError(e)),
        }
    }

    pub fn validate(self) -> Result<RoomTemplate, RoomTemplateError> {
        let mut unmapping = Vec::new();

        self.unmapping
            .iter()
            .for_each(|(key, _value)| unmapping.push(key.clone()));

        for object in &self.objects {
            object
                .unmapping
                .iter()
                .for_each(|(key, _value)| unmapping.push(format!("object/{}", key)));
            object
                .fields
                .unmapping
                .iter()
                .for_each(|(key, _value)| unmapping.push(format!("object/fields/{}", key)));
        }

        if unmapping.is_empty() {
            Result::Ok(self)
        } else {
            Result::Err(RoomTemplateError::YamlContainsUnmappingFields(unmapping))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::service::yaml::{GameObjectTemplate, RoomTemplate, RoomTemplateError};

    #[test]
    fn should_fail_if_unmapping_field() {
        let mut template = RoomTemplate::default();
        template
            .unmapping
            .insert("wrong_field".to_string(), serde_yaml::Value::default());

        let mut object_template = GameObjectTemplate {
            id: 0,
            template: 0,
            access_groups: Default::default(),
            fields: Default::default(),
            unmapping: Default::default(),
        };
        object_template
            .unmapping
            .insert("wrong_field".to_string(), serde_yaml::Value::default());
        object_template
            .fields
            .unmapping
            .insert("wrong_field".to_string(), serde_yaml::Value::default());

        template.objects = Default::default();
        template.objects.push(object_template);

        assert!(matches!(
            template.validate(),
            Result::Err(RoomTemplateError::YamlContainsUnmappingFields(fields))
            if fields[0] == "wrong_field"
            && fields[1] == "object/wrong_field"
            && fields[2] == "object/fields/wrong_field"
        ))
    }
}
