use std::collections::HashMap;

use serde::{Deserialize, Serialize};

///
/// Структуры для чтения шаблона комнаты из yaml файла
/// Специально разделен с grpc типами, так как yaml представление может иметь другую структуру
/// для большего удобства
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RoomTemplate {
    #[serde(default)]
    pub objects: Vec<GameObjectTemplate>,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(flatten)]
    pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Permissions {
    pub templates: Vec<GameObjectTemplatePermission>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectTemplate {
    pub id: u32,
    pub template: u16,
    pub groups: u64,
    #[serde(default)]
    pub fields: GameObjectFieldsTemplate,
    #[serde(flatten)]
    pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectFieldsTemplate {
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
pub struct GameObjectTemplatePermission {
    pub template: u16,
    #[serde(default)]
    pub rules: Vec<GroupsPermissionRule>,
    #[serde(default)]
    pub fields: Vec<PermissionField>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct GroupsPermissionRule {
    pub groups: u64,
    pub permission: PermissionLevel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionField {
    pub id: u16,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub rules: Vec<GroupsPermissionRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum PermissionLevel {
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
    pub fn new_from_yaml(content: &str) -> Result<RoomTemplate, RoomTemplateError> {
        let template = serde_yaml::from_str::<RoomTemplate>(content);
        match template {
            Ok(template) => template.validate(),
            Err(e) => Result::Err(RoomTemplateError::YamlParserError(e)),
        }
    }

    fn validate(self) -> Result<RoomTemplate, RoomTemplateError> {
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
    use crate::service::yaml::{
        FieldType, GameObjectFieldsTemplate, GameObjectTemplate, GameObjectTemplatePermission,
        GroupsPermissionRule, PermissionField, PermissionLevel, Permissions, RoomTemplate,
        RoomTemplateError,
    };
    use rmpv::Integer;
    use rmpv::Utf8String;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[test]
    fn should_fail_if_unmapping_field() {
        let mut template = RoomTemplate::default();
        template
            .unmapping
            .insert("wrong_field".to_string(), serde_yaml::Value::default());

        let mut object_template = GameObjectTemplate {
            id: 0,
            template: 0,
            groups: Default::default(),
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

    #[test]
    fn dump_yaml_for_docs() {
        let yaml = RoomTemplate {
            objects: vec![GameObjectTemplate {
                id: 555,
                template: 1,
                groups: 4857,
                fields: GameObjectFieldsTemplate {
                    longs: HashMap::from_iter(vec![(10, 100100)].into_iter()),
                    floats: HashMap::from_iter(vec![(5, 3.14), (10, 2.68)].into_iter()),
                    structures: HashMap::from_iter(
                        vec![
                            (
                                5,
                                rmpv::Value::Map(vec![
                                    (
                                        rmpv::Value::String(Utf8String::from("uid".to_owned())),
                                        rmpv::Value::String(Utf8String::from("arts80").to_owned()),
                                    ),
                                    (
                                        rmpv::Value::String(Utf8String::from("rank".to_owned())),
                                        rmpv::Value::Integer(Integer::from(100)),
                                    ),
                                ]),
                            ),
                            (
                                10,
                                rmpv::Value::Array(vec![
                                    rmpv::Value::Integer(Integer::from(15)),
                                    rmpv::Value::Integer(Integer::from(26)),
                                ]),
                            ),
                        ]
                        .into_iter(),
                    ),
                    unmapping: Default::default(),
                },
                unmapping: Default::default(),
            }],
            permissions: Permissions {
                templates: vec![GameObjectTemplatePermission {
                    template: 1,
                    rules: vec![GroupsPermissionRule {
                        groups: 12495,
                        permission: PermissionLevel::Deny,
                    }],
                    fields: vec![PermissionField {
                        id: 100,
                        field_type: FieldType::Long,
                        rules: vec![GroupsPermissionRule {
                            groups: 5677,
                            permission: PermissionLevel::Ro,
                        }],
                    }],
                }],
            },
            unmapping: Default::default(),
        };
        println!("{}", serde_yaml::to_string(&yaml).unwrap());
    }
}
