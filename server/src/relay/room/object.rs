use std::collections::HashMap;
use std::iter::Map;
use std::ops::Shl;

use crate::relay::room::groups::AccessGroups;

type DataId = u32;


/// Игровой объект
/// содержит данные от пользователей
pub struct GameObject {
    pub id: u64,
    pub owner: u16,
    /// счетчики
    counters: HashMap<DataId, DataCounter>,
    /// структуры (для сервера это массивы данных)
    structs: HashMap<DataId, DataStruct>,
    /// группы доступа
    pub groups: AccessGroups,
}

/// счетчик
pub struct DataCounter {
    counter: i64
}

/// данные
pub struct DataStruct {
    data: Box<[u8]>
}

impl GameObject {
    pub fn new(owner: u16, local_id: u32, groups: AccessGroups) -> GameObject {
        GameObject {
            id: (owner as u64).shl(32) + local_id as u64,
            owner,
            counters: Default::default(),
            structs: Default::default(),
            groups,
        }
    }
}

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