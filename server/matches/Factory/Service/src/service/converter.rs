use crate::proto::matches::relay::types as grpc;
use crate::service::yaml;

impl Into<grpc::RoomTemplate> for yaml::RoomTemplate {
    fn into(self) -> grpc::RoomTemplate {
        grpc::RoomTemplate {
            objects: self.objects.into_iter().map(Into::into).collect(),
            template_permissions: self.permissions.into_iter().map(Into::into).collect(),
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

impl Into<grpc::FieldsTemplate> for yaml::FieldsTemplate {
    fn into(self) -> grpc::FieldsTemplate {
        grpc::FieldsTemplate {
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
impl Into<grpc::TemplatePermission> for yaml::TemplatePermission {
    fn into(self) -> grpc::TemplatePermission {
        grpc::TemplatePermission {
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

impl Into<grpc::PermissionGroup> for yaml::PermissionGroup {
    fn into(self) -> grpc::PermissionGroup {
        grpc::PermissionGroup {
            access_group: self.group,
            permission: self.permission.into(),
        }
    }
}
impl Into<i32> for yaml::Permission {
    fn into(self) -> i32 {
        match self {
            yaml::Permission::Deny => grpc::Permission::Deny as i32,
            yaml::Permission::Ro => grpc::Permission::Ro as i32,
            yaml::Permission::Rw => grpc::Permission::Rw as i32,
        }
    }
}
