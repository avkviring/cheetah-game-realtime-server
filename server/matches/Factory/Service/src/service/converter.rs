use crate::proto::matches::relay::types as relay;
use crate::service::yaml;

///
/// Конвертация yaml представления шаблона комнаты в grpc представление
///
impl Into<relay::RoomTemplate> for yaml::RoomTemplate {
    fn into(self) -> relay::RoomTemplate {
        relay::RoomTemplate {
            objects: self.objects.into_iter().map(Into::into).collect(),
            permissions: Option::Some(self.permissions.into()),
        }
    }
}

impl Into<relay::Permissions> for yaml::Permissions {
    fn into(self) -> relay::Permissions {
        relay::Permissions {
            objects: self.templates.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<relay::GameObjectTemplate> for yaml::GameObjectTemplate {
    fn into(self) -> relay::GameObjectTemplate {
        relay::GameObjectTemplate {
            id: self.id,
            template: self.template as u32,
            groups: self.groups,
            fields: Option::Some(self.fields.into()),
        }
    }
}

impl Into<relay::GameObjectFieldsTemplate> for yaml::GameObjectFieldsTemplate {
    fn into(self) -> relay::GameObjectFieldsTemplate {
        relay::GameObjectFieldsTemplate {
            longs: self.longs.into_iter().map(|(k, v)| (k as u32, v)).collect(),
            floats: self
                .floats
                .into_iter()
                .map(|(k, v)| (k as u32, v))
                .collect(),
            structures: self
                .structures
                .into_iter()
                .map(|(k, v)| (k as u32, rmp_serde::to_vec(&v).unwrap()))
                .collect(),
        }
    }
}
impl Into<relay::GameObjectTemplatePermission> for yaml::GameObjectTemplatePermission {
    fn into(self) -> relay::GameObjectTemplatePermission {
        relay::GameObjectTemplatePermission {
            template: self.template as u32,
            rules: self.rules.into_iter().map(Into::into).collect(),
            fields: self.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<relay::PermissionField> for yaml::PermissionField {
    fn into(self) -> relay::PermissionField {
        relay::PermissionField {
            id: self.id as u32,
            r#type: self.field_type.into(),
            rules: self.rules.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<i32> for yaml::FieldType {
    fn into(self) -> i32 {
        match self {
            yaml::FieldType::Long => relay::FieldType::Long as i32,
            yaml::FieldType::Float => relay::FieldType::Float as i32,
            yaml::FieldType::Structure => relay::FieldType::Structure as i32,
            yaml::FieldType::Event => relay::FieldType::Event as i32,
        }
    }
}

impl Into<relay::GroupsPermissionRule> for yaml::GroupsPermissionRule {
    fn into(self) -> relay::GroupsPermissionRule {
        relay::GroupsPermissionRule {
            groups: self.groups,
            permission: self.permission.into(),
        }
    }
}
impl Into<i32> for yaml::PermissionLevel {
    fn into(self) -> i32 {
        match self {
            yaml::PermissionLevel::Deny => relay::PermissionLevel::Deny as i32,
            yaml::PermissionLevel::Ro => relay::PermissionLevel::Ro as i32,
            yaml::PermissionLevel::Rw => relay::PermissionLevel::Rw as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use rmpv::{Integer, Utf8String};

    use crate::proto::matches::relay::types as relay;
    use crate::service::yaml;
    use crate::service::yaml::{
        FieldType, GameObjectFieldsTemplate, GroupsPermissionRule, PermissionField, PermissionLevel,
    };

    #[test]
    fn should_convert_room_template() {
        let yaml = yaml::RoomTemplate {
            objects: vec![yaml::GameObjectTemplate::default()],
            permissions: yaml::Permissions {
                templates: vec![yaml::GameObjectTemplatePermission::default()],
            },
            unmapping: Default::default(),
        };
        let grpc: relay::RoomTemplate = yaml.into();
        assert_eq!(grpc.objects.len(), 1);
        assert_eq!(grpc.permissions.as_ref().unwrap().objects.len(), 1);
    }
    #[test]
    fn should_convert_game_object_template() {
        let yaml_object_template = yaml::GameObjectTemplate {
            id: 100,
            template: 200,
            groups: 300,
            fields: yaml::GameObjectFieldsTemplate::default(),
            unmapping: Default::default(),
        };
        let grpc_object_template: relay::GameObjectTemplate = yaml_object_template.clone().into();
        assert_eq!(grpc_object_template.id, yaml_object_template.id);
        assert_eq!(
            grpc_object_template.template as u16,
            yaml_object_template.template
        );
        assert_eq!(grpc_object_template.groups, yaml_object_template.groups);
        assert_eq!(grpc_object_template.fields.is_some(), true);
    }

    #[test]
    fn should_convert_fields() {
        let yaml_item = GameObjectFieldsTemplate {
            longs: vec![(10, 20)].into_iter().collect(),
            floats: vec![(15, 30.30)].into_iter().collect(),
            structures: vec![(15, rmpv::Value::Binary(vec![10, 20, 30]))]
                .into_iter()
                .collect(),
            unmapping: Default::default(),
        };
        let grpc_item: relay::GameObjectFieldsTemplate = yaml_item.clone().into();
        let grpc_longs: HashMap<u16, i64> = grpc_item
            .longs
            .clone()
            .into_iter()
            .map(|(k, v)| (k as u16, v))
            .collect();
        let yaml_longs = yaml_item.longs.clone();
        assert_eq!(grpc_longs, yaml_longs);

        let grpc_floats: HashMap<_, _> = grpc_item
            .floats
            .clone()
            .into_iter()
            .map(|(k, v)| (k as u16, v))
            .collect();
        let yaml_floats = yaml_item.floats.clone();
        assert_eq!(grpc_floats, yaml_floats);

        let grpc_structures: HashMap<_, _> = grpc_item
            .structures
            .clone()
            .into_iter()
            .map(|(k, v)| (k as u16, rmp_serde::from_read(v.as_slice()).unwrap()))
            .collect();
        let yaml_structures = yaml_item.structures.clone();
        assert_eq!(grpc_structures, yaml_structures);
    }

    #[test]
    fn should_convert_permissions() {
        let yaml_item = yaml::Permissions {
            templates: vec![yaml::GameObjectTemplatePermission::default()],
        };

        let grpc_item: relay::Permissions = yaml_item.into();
        assert_eq!(grpc_item.objects.len(), 1);
    }

    #[test]
    fn should_convert_game_object_template_permission() {
        let yaml_item = yaml::GameObjectTemplatePermission {
            template: 10,
            rules: vec![yaml::GroupsPermissionRule {
                groups: 0,
                permission: yaml::PermissionLevel::Deny,
            }],
            fields: vec![yaml::PermissionField {
                id: 0,
                field_type: yaml::FieldType::Long,
                rules: vec![],
            }],
        };

        let grpc_item: relay::GameObjectTemplatePermission = yaml_item.clone().into();
        assert_eq!(grpc_item.template as u16, yaml_item.template);
        assert_eq!(grpc_item.rules.len(), 1);
        assert_eq!(grpc_item.fields.len(), 1);
    }

    #[test]
    fn should_convert_access_group_permission_level() {
        let yaml_item = yaml::GroupsPermissionRule {
            groups: 10,
            permission: yaml::PermissionLevel::Deny,
        };
        let grpc_item: relay::GroupsPermissionRule = yaml_item.clone().into();
        assert_eq!(grpc_item.groups, yaml_item.groups);
        assert_eq!(grpc_item.permission, relay::PermissionLevel::Deny as i32);
    }

    #[test]
    fn should_convert_permission_field() {
        let yaml_item = yaml::PermissionField {
            id: 55,
            field_type: yaml::FieldType::Long,
            rules: vec![yaml::GroupsPermissionRule {
                groups: 0,
                permission: yaml::PermissionLevel::Deny,
            }],
        };
        let grpc_item: relay::PermissionField = yaml_item.clone().into();
        assert_eq!(grpc_item.id as u16, yaml_item.id);
        assert_eq!(grpc_item.r#type, relay::FieldType::Long as i32);
        assert_eq!(grpc_item.rules.len(), 1);
    }
}
