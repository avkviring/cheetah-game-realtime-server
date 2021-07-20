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
            objects: self.objects.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<relay::GameObjectTemplate> for yaml::GameObjectTemplate {
    fn into(self) -> relay::GameObjectTemplate {
        relay::GameObjectTemplate {
            id: self.id,
            template: self.template as u32,
            access_group: self.access_groups,
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
            groups: self.groups.into_iter().map(Into::into).collect(),
            fields: self.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<relay::PermissionField> for yaml::PermissionField {
    fn into(self) -> relay::PermissionField {
        relay::PermissionField {
            field_id: self.field_id as u32,
            field_type: self.field_type.into(),
            groups: self.groups.into_iter().map(Into::into).collect(),
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

impl Into<relay::AccessGroupPermissionLevel> for yaml::AccessGroupPermissionLevel {
    fn into(self) -> relay::AccessGroupPermissionLevel {
        relay::AccessGroupPermissionLevel {
            access_group: self.access_group,
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
        AccessGroupPermissionLevel, FieldType, GameObjectFieldsTemplate, PermissionField,
        PermissionLevel,
    };

    #[test]
    fn should_convert_room_template() {
        let yaml = yaml::RoomTemplate {
            objects: vec![yaml::GameObjectTemplate::default()],
            permissions: yaml::Permissions {
                objects: vec![yaml::GameObjectTemplatePermission::default()],
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
            access_groups: 300,
            fields: yaml::GameObjectFieldsTemplate::default(),
            unmapping: Default::default(),
        };
        let grpc_object_template: relay::GameObjectTemplate = yaml_object_template.clone().into();
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
            objects: vec![yaml::GameObjectTemplatePermission::default()],
        };

        let grpc_item: relay::Permissions = yaml_item.into();
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

        let grpc_item: relay::GameObjectTemplatePermission = yaml_item.clone().into();
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
        let grpc_item: relay::AccessGroupPermissionLevel = yaml_item.clone().into();
        assert_eq!(grpc_item.access_group, yaml_item.access_group);
        assert_eq!(grpc_item.permission, relay::PermissionLevel::Deny as i32);
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
        let grpc_item: relay::PermissionField = yaml_item.clone().into();
        assert_eq!(grpc_item.field_id as u16, yaml_item.field_id);
        assert_eq!(grpc_item.field_type, relay::FieldType::Long as i32);
        assert_eq!(grpc_item.groups.len(), 1);
    }

    #[test]
    fn dump_yaml_for_docs() {
        let yaml = yaml::RoomTemplate {
            objects: vec![yaml::GameObjectTemplate {
                id: 555,
                template: 1,
                access_groups: 4857,
                fields: yaml::GameObjectFieldsTemplate {
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
            permissions: yaml::Permissions {
                objects: vec![yaml::GameObjectTemplatePermission {
                    template: 1,
                    groups: vec![AccessGroupPermissionLevel {
                        access_group: 12495,
                        permission: PermissionLevel::Deny,
                    }],
                    fields: vec![PermissionField {
                        field_id: 100,
                        field_type: FieldType::Long,
                        groups: vec![AccessGroupPermissionLevel {
                            access_group: 5677,
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
