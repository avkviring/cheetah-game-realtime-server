use crate::proto::matches::relay::types as grpc;
use crate::service::yaml;

///
/// Конвертация yaml представления шаблона комнаты в grpc представление
///
impl Into<grpc::RoomTemplate> for yaml::RoomTemplate {
    fn into(self) -> grpc::RoomTemplate {
        grpc::RoomTemplate {
            objects: self.objects.into_iter().map(Into::into).collect(),
            permissions: Option::Some(self.permissions.into()),
        }
    }
}

impl Into<grpc::Permissions> for yaml::Permissions {
    fn into(self) -> grpc::Permissions {
        grpc::Permissions {
            objects: self.objects.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<grpc::GameObjectTemplate> for yaml::GameObjectTemplate {
    fn into(self) -> grpc::GameObjectTemplate {
        grpc::GameObjectTemplate {
            id: self.id,
            template: self.template as u32,
            access_group: self.access_groups,
            fields: Option::Some(self.fields.into()),
        }
    }
}

impl Into<grpc::GameObjectFieldsTemplate> for yaml::GameObjectFieldsTemplate {
    fn into(self) -> grpc::GameObjectFieldsTemplate {
        grpc::GameObjectFieldsTemplate {
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
impl Into<grpc::GameObjectTemplatePermission> for yaml::GameObjectTemplatePermission {
    fn into(self) -> grpc::GameObjectTemplatePermission {
        grpc::GameObjectTemplatePermission {
            template: self.template as u32,
            groups: self.groups.into_iter().map(Into::into).collect(),
            fields: self.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<grpc::PermissionField> for yaml::PermissionField {
    fn into(self) -> grpc::PermissionField {
        grpc::PermissionField {
            field_id: self.field_id as u32,
            field_type: self.field_type.into(),
            groups: self.groups.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<i32> for yaml::FieldType {
    fn into(self) -> i32 {
        match self {
            yaml::FieldType::Long => grpc::FieldType::Long as i32,
            yaml::FieldType::Float => grpc::FieldType::Float as i32,
            yaml::FieldType::Structure => grpc::FieldType::Structure as i32,
            yaml::FieldType::Event => grpc::FieldType::Event as i32,
        }
    }
}

impl Into<grpc::AccessGroupPermissionLevel> for yaml::AccessGroupPermissionLevel {
    fn into(self) -> grpc::AccessGroupPermissionLevel {
        grpc::AccessGroupPermissionLevel {
            access_group: self.access_group,
            permission: self.permission.into(),
        }
    }
}
impl Into<i32> for yaml::PermissionLevel {
    fn into(self) -> i32 {
        match self {
            yaml::PermissionLevel::Deny => grpc::PermissionLevel::Deny as i32,
            yaml::PermissionLevel::Ro => grpc::PermissionLevel::Ro as i32,
            yaml::PermissionLevel::Rw => grpc::PermissionLevel::Rw as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::matches::relay::types as grpc;
    use crate::service::yaml;
    use crate::service::yaml::{GameObjectFieldsTemplate, Permissions};
    use std::collections::HashMap;

    #[test]
    fn should_convert_room_template() {
        let yaml = yaml::RoomTemplate {
            objects: vec![yaml::GameObjectTemplate::default()],
            permissions: yaml::Permissions {
                objects: vec![yaml::GameObjectTemplatePermission::default()],
            },
            unmapping: Default::default(),
        };
        let grpc: grpc::RoomTemplate = yaml.into();
        assert_eq!(grpc.objects.len(), 1);
        assert_eq!(grpc.permissions.as_ref().unwrap().objects.len(), 1);
    }
    #[test]
    fn should_convert_game_object_template() {
        let yaml_object_template = yaml::GameObjectTemplate {
            id: 100,
            template: 200,
            access_groups: 300,
            fields: yaml::GameObjectFieldsTemplate::default(),
            unmapping: Default::default(),
        };
        let grpc_object_template: grpc::GameObjectTemplate = yaml_object_template.clone().into();
        assert_eq!(grpc_object_template.id, yaml_object_template.id);
        assert_eq!(
            grpc_object_template.template as u16,
            yaml_object_template.template
        );
        assert_eq!(
            grpc_object_template.access_group,
            yaml_object_template.access_groups
        );
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
        let grpc_item: grpc::GameObjectFieldsTemplate = yaml_item.clone().into();
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
            objects: vec![yaml::GameObjectTemplatePermission::default()],
        };

        let grpc_item: grpc::Permissions = yaml_item.into();
        assert_eq!(grpc_item.objects.len(), 1);
    }

    #[test]
    fn should_convert_game_object_template_permission() {
        let yaml_item = yaml::GameObjectTemplatePermission {
            template: 10,
            groups: vec![yaml::AccessGroupPermissionLevel {
                access_group: 0,
                permission: yaml::PermissionLevel::Deny,
            }],
            fields: vec![yaml::PermissionField {
                field_id: 0,
                field_type: yaml::FieldType::Long,
                groups: vec![],
            }],
        };

        let grpc_item: grpc::GameObjectTemplatePermission = yaml_item.clone().into();
        assert_eq!(grpc_item.template as u16, yaml_item.template);
        assert_eq!(grpc_item.groups.len(), 1);
        assert_eq!(grpc_item.fields.len(), 1);
    }

    #[test]
    fn should_convert_access_group_permission_level() {
        let yaml_item = yaml::AccessGroupPermissionLevel {
            access_group: 10,
            permission: yaml::PermissionLevel::Deny,
        };
        let grpc_item: grpc::AccessGroupPermissionLevel = yaml_item.clone().into();
        assert_eq!(grpc_item.access_group, yaml_item.access_group);
        assert_eq!(grpc_item.permission, grpc::PermissionLevel::Deny as i32);
    }

    #[test]
    fn should_convert_permission_field() {
        let yaml_item = yaml::PermissionField {
            field_id: 55,
            field_type: yaml::FieldType::Long,
            groups: vec![yaml::AccessGroupPermissionLevel {
                access_group: 0,
                permission: yaml::PermissionLevel::Deny,
            }],
        };
        let grpc_item: grpc::PermissionField = yaml_item.clone().into();
        assert_eq!(grpc_item.field_id as u16, yaml_item.field_id);
        assert_eq!(grpc_item.field_type, grpc::FieldType::Long as i32);
        assert_eq!(grpc_item.groups.len(), 1);
    }
}
