use super::{Error, ExtendField, FieldValue, GroupResolver, ObjectField, OptionValue, Prefab};
use crate::proto::matches::relay::types as relay;
use std::collections::HashMap;
use std::path::Path;

pub struct PrefabResolver {
    template: relay::GameObjectTemplatePermission,
    defaults: HashMap<u32, FieldValue>,
    field_names: HashMap<String, u32>,
}

impl PrefabResolver {
    pub fn template_id(&self) -> u32 {
        self.template.template
    }

    pub fn template(&self) -> relay::GameObjectTemplatePermission {
        self.template.clone()
    }

    pub fn new(prefab: Prefab, groups: &GroupResolver, path: &Path) -> Result<Self, Error> {
        let rules = prefab
            .access
            .into_iter()
            .map(|(name, rule)| groups.resolve(&name, rule, path))
            .collect::<Result<_, Error>>()?;

        let field_names: HashMap<String, u32> = prefab
            .fields
            .iter()
            .map(|field| (field.name.clone(), field.id))
            .collect();

        let defaults = prefab
            .fields
            .iter()
            .filter_map(|field| {
                field
                    .field
                    .clone()
                    .into_value()
                    .map(|value| (field_names[&field.name], value))
            })
            .collect();

        let fields = prefab
            .fields
            .into_iter()
            .map(|field| {
                let id = field_names[&field.name];

                let r#type = match field.field {
                    OptionValue::I64 { .. } => relay::FieldType::Long as i32,
                    OptionValue::F64 { .. } => relay::FieldType::Float as i32,
                    OptionValue::Struct { .. } => relay::FieldType::Structure as i32,
                    OptionValue::Event { .. } => relay::FieldType::Event as i32,
                };

                let rules = field
                    .access
                    .into_iter()
                    .map(|(name, rule)| groups.resolve(&name, rule, path))
                    .collect::<Result<_, Error>>()?;

                Ok(relay::PermissionField { id, r#type, rules })
            })
            .collect::<Result<_, Error>>()?;

        let template = relay::GameObjectTemplatePermission {
            template: prefab.template,
            rules,
            fields,
        };

        Ok(Self {
            template,
            defaults,
            field_names,
        })
    }

    pub fn resolve(
        &self,
        base: Vec<ObjectField>,
        extend: Vec<ExtendField>,
        path: &Path,
    ) -> Result<relay::GameObjectFieldsTemplate, Error> {
        let mut fields = self.defaults.clone();

        for ObjectField { name, value } in base {
            let id = self
                .field_names
                .get(&name)
                .copied()
                .ok_or_else(|| Error::PrefabFieldNotExists(path.into(), name.clone()))?;

            fields.insert(id, value);
        }

        for ExtendField { id, value } in extend {
            if fields.insert(id, value).is_some() {
                return Err(Error::ObjectFieldExists(path.into(), id));
            }
        }

        let longs = fields.iter().filter_map(|(&key, field)| match field {
            FieldValue::I64 { value } => Some((key, *value)),
            _ => None,
        });
        let floats = fields.iter().filter_map(|(&key, field)| match field {
            FieldValue::F64 { value } => Some((key, *value)),
            _ => None,
        });
        let structures = fields.iter().filter_map(|(&key, field)| match field {
            FieldValue::Struct { value } => Some((key, rmp_serde::to_vec(&value).ok()?)),
            _ => None,
        });

        Ok(relay::GameObjectFieldsTemplate {
            longs: longs.collect(),
            floats: floats.collect(),
            structures: structures.collect(),
        })
    }
}

#[cfg(test)]
#[test]
fn resolver() {
    use super::{PrefabField, Rule};

    let groups = {
        let mut groups = HashMap::new();
        groups.insert(Path::new("/dir/groups").into(), {
            let mut file = HashMap::default();
            file.insert("test".into(), 12345);
            file
        });
        GroupResolver::build(groups).1
    };

    let mut access = HashMap::default();
    access.insert("test".into(), Rule::Deny);

    let prefab = Prefab {
        template: 4444,
        groups: "/dir/groups".into(),
        access: access.clone(),
        fields: vec![
            PrefabField {
                name: "a".to_string(),
                id: 1,
                access: access.clone(),
                field: OptionValue::I64 { value: Some(7) },
            },
            PrefabField {
                name: "b".to_string(),
                id: 2,
                access: access.clone(),
                field: OptionValue::I64 { value: None },
            },
            PrefabField {
                name: "default".to_string(),
                id: 3,
                access: access.clone(),
                field: OptionValue::I64 { value: Some(22222) },
            },
        ],
    };

    let resolver = PrefabResolver::new(prefab, &groups, Path::new("")).unwrap();

    assert_eq!(resolver.template_id(), 4444);

    {
        let base = vec![
            ObjectField {
                name: "a".into(),
                value: FieldValue::I64 { value: 12345 },
            },
            ObjectField {
                name: "b".into(),
                value: FieldValue::I64 { value: 77777 },
            },
        ];

        let extend = vec![ExtendField {
            id: 4321,
            value: FieldValue::I64 { value: 99999 },
        }];

        let obj = resolver.resolve(base, extend, Path::new("")).unwrap();

        assert_eq!(obj.longs[&1], 12345); // перезаписано
        assert_eq!(obj.longs[&2], 77777); // установлено значение
        assert_eq!(obj.longs[&3], 22222); // взято из префаба
        assert_eq!(obj.longs[&4321], 99999); // добавлено из объекта
    }
}
