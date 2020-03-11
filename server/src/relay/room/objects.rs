use std::collections::HashMap;

use crate::relay::room::object::GameObject;

/// Хранение и управление списком игровых объектов
pub struct Objects {
    objects: HashMap<u64, GameObject>,
}

impl Objects {
    pub fn new() -> Objects {
        Objects {
            objects: Default::default()
        }
    }

    pub fn insert(&mut self, object: GameObject) {
        self.objects.insert(object.id, object);
    }

    pub fn get(&mut self, id: u64) -> Option<&GameObject> {
        return self.objects.get(&id);
    }

    pub fn len(&mut self) -> usize {
        return self.objects.len();
    }

    pub fn remove_objects_by_owner(&mut self, owner: u16) {
        let object_for_remove: Vec<u64> = self.objects
            .values()
            .filter(|o| o.owner == owner)
            .map(|f| f.id)
            .collect();


        for object_id in object_for_remove {
            self.objects.remove(&object_id);
        }
    }
}

impl Default for Objects {
    fn default() -> Self {
        Objects {
            objects: Default::default(),
        }
    }
}


