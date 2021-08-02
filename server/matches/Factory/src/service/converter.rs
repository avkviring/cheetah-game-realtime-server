use crate::proto::matches::relay::types as relay;
use crate::service::room;
use std::collections::HashMap;

/// Конвертация yaml представления шаблона комнаты в grpc представление
impl From<room::Room> for relay::RoomTemplate {
    fn from(room::Room { objects, templates }: room::Room) -> Self {
        relay::RoomTemplate {
            objects: objects.into_iter().map(Into::into).collect(),
            permissions: Some(templates.into()),
        }
    }
}

impl From<HashMap<u32, room::Permissions>> for relay::Permissions {
    fn from(val: HashMap<u32, room::Permissions>) -> Self {
        relay::Permissions {
            objects: val.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<(u32, room::GameObject)> for relay::GameObjectTemplate {
    fn from((id, val): (u32, room::GameObject)) -> Self {
        relay::GameObjectTemplate {
            id,
            template: val.template,
            groups: val.groups,
            fields: Some(val.fields.into()),
        }
    }
}

impl From<HashMap<u32, room::ObjectField>> for relay::GameObjectFieldsTemplate {
    fn from(val: HashMap<u32, room::ObjectField>) -> Self {
        let longs = val.iter().filter_map(|(&key, value)| match value {
            room::ObjectField::I64 { value } => Some((key, *value)),
            _ => None,
        });

        let floats = val.iter().filter_map(|(&key, value)| match value {
            room::ObjectField::F64 { value } => Some((key, *value)),
            _ => None,
        });

        let structures = val.iter().filter_map(|(&key, value)| match value {
            room::ObjectField::Struct { value } => Some((key, rmp_serde::to_vec(value).unwrap())),
            _ => None,
        });

        Self {
            longs: longs.collect(),
            floats: floats.collect(),
            structures: structures.collect(),
        }
    }
}

impl From<(u32, room::Permissions)> for relay::GameObjectTemplatePermission {
    fn from((template, room::Permissions { rules, fields }): (u32, room::Permissions)) -> Self {
        relay::GameObjectTemplatePermission {
            template,
            rules: rules.into_iter().map(Into::into).collect(),
            fields: fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<(u32, room::PermissionField)> for relay::PermissionField {
    fn from((id, field): (u32, room::PermissionField)) -> Self {
        let (r#type, rules) = match field {
            room::PermissionField::I64 { rules } => (relay::FieldType::Long, rules),
            room::PermissionField::F64 { rules } => (relay::FieldType::Float, rules),
            room::PermissionField::Struct { rules } => (relay::FieldType::Structure, rules),
            room::PermissionField::Event { rules } => (relay::FieldType::Event, rules),
        };
        Self {
            id,
            r#type: r#type as i32,
            rules: rules.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<room::Rule> for relay::GroupsPermissionRule {
    fn from(val: room::Rule) -> Self {
        let (groups, permission) = match val {
            room::Rule::Deny { groups } => (groups, relay::PermissionLevel::Deny as i32),
            room::Rule::ReadOnly { groups } => (groups, relay::PermissionLevel::Ro as i32),
            room::Rule::ReadWrite { groups } => (groups, relay::PermissionLevel::Rw as i32),
        };
        Self { groups, permission }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::matches::relay::types as relay;
    use crate::service::room;
    use std::collections::HashMap;

    #[test]
    fn should_convert_room_template() {
        let yaml = room::Room {
            objects: std::iter::once((0, room::GameObject::default())).collect(),
            templates: std::iter::once((0, room::Permissions::default())).collect(),
        };
        let grpc: relay::RoomTemplate = yaml.into();
        assert_eq!(grpc.objects.len(), 1);
        assert_eq!(grpc.permissions.as_ref().unwrap().objects.len(), 1);
    }

    #[test]
    fn should_convert_game_object_template() {
        let object = room::GameObject {
            template: 200,
            groups: 300,
            fields: HashMap::default(),
        };
        let grpc: relay::GameObjectTemplate = (100, object.clone()).into();
        assert_eq!(grpc.id, 100);
        assert_eq!(grpc.template, object.template);
        assert_eq!(grpc.groups, object.groups);
        assert!(matches!(grpc.fields, Some(_)));
    }

    #[test]
    fn should_convert_fields() {
        let long_item = room::ObjectField::I64 { value: 20 };
        let float_item = room::ObjectField::F64 { value: 30.30 };
        let struct_value = rmpv::Value::Binary(vec![10, 20, 30]);
        let struct_item = room::ObjectField::Struct {
            value: struct_value.clone(),
        };

        let items: HashMap<_, _> =
            IntoIterator::into_iter([(1, long_item), (2, float_item), (3, struct_item)]).collect();

        let grpc: relay::GameObjectFieldsTemplate = items.into();
        assert_eq!(grpc.longs[&1], 20);
        assert_eq!(grpc.floats[&2], 30.30);
        assert_eq!(
            rmp_serde::from_read::<_, rmpv::Value>(grpc.structures[&3].as_slice()).unwrap(),
            struct_value
        );
    }

    #[test]
    fn should_convert_permissions() {
        let yaml_item: HashMap<_, _> = std::iter::once((0, room::Permissions::default())).collect();

        let grpc_item: relay::Permissions = yaml_item.into();
        assert_eq!(grpc_item.objects.len(), 1);
    }

    #[test]
    fn should_convert_game_object_template_permission() {
        let yaml_item = room::Permissions {
            rules: vec![room::Rule::Deny { groups: 0 }],
            fields: std::iter::once((0, room::PermissionField::I64 { rules: vec![] })).collect(),
        };

        let grpc_item: relay::GameObjectTemplatePermission = (10, yaml_item).into();
        assert_eq!(grpc_item.template, 10);
        assert_eq!(grpc_item.rules.len(), 1);
        assert_eq!(grpc_item.fields.len(), 1);
    }

    #[test]
    fn should_convert_access_group_permission_level() {
        let groups = 10;
        let yaml_item = room::Rule::Deny { groups };
        let grpc_item: relay::GroupsPermissionRule = yaml_item.into();
        assert_eq!(grpc_item.groups, groups);
        assert_eq!(grpc_item.permission, relay::PermissionLevel::Deny as i32);
    }
}
