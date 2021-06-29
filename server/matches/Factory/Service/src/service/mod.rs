use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub mod grpc;

pub struct FactoryService {
    templates: HashMap<String, crate::proto::relay::RoomTemplate>,
}

impl FactoryService {
    pub fn new(path: &Path) -> Self {
        let templates = load_templates(path);
        FactoryService { templates }
    }
}

fn load_templates(path: &Path) -> HashMap<String, crate::proto::relay::RoomTemplate> {
    let mut result = HashMap::default();
    let r: Vec<()> = fs::read_dir(path)
        .unwrap()
        .map(|res| {
            let res = res.unwrap();
            println!("{:?}", res);
            ()
        })
        .collect();

    result
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;

    use crate::service::load_templates;
    use std::time::Duration;

    #[test]
    pub fn load_templates_test() {
        let templates_directory = tempfile::TempDir::new().unwrap();
        let room_template = r#"
        objects:
         - id: 5
           template: 5
           access_groups: 0
           fields:
            longs:
                10: 1020
        "#;
        {
            let mut room_file = File::create(templates_directory.path().join("room.yaml")).unwrap();
            room_file.write_all(room_template.as_bytes()).unwrap();
            room_file.sync_all().unwrap();
        }
        load_templates(templates_directory.path());
    }
}
