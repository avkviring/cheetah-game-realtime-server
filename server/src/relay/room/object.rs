use std::collections::HashMap;
use std::iter::Map;
use std::ops::Shl;

use bytes::Bytes;

use crate::relay::room::groups::AccessGroups;

/// Игровой объект
/// содержит данные от пользователей
pub struct GameObject {
    pub id: u64,
    pub owner: u16,
    /// счетчики
    counters: HashMap<u16, DataCounter>,
    /// структуры (для сервера это массивы данных)
    structs: HashMap<u16, DataStruct>,
    /// группы доступа
    pub groups: AccessGroups,
}

/// счетчик
pub struct DataCounter {
    counter: i64
}

/// данные
pub struct DataStruct {
    data: Bytes
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

    pub fn update_struct(&mut self, struct_id: u16, data: &[u8]) {
        self.structs.insert(struct_id, DataStruct { data: Bytes::copy_from_slice(data) });
    }

    pub fn get_struct(&self, struct_id: u16) -> Option<&Bytes> {
        self.structs.get(&struct_id).map(|f| &f.data)
    }

    pub fn set_counter(&mut self, counter_id: u16, value: i64) {
        self.counters.insert(counter_id, DataCounter { counter: value });
    }

    pub fn get_counter(&mut self, counter_id: u16) -> i64 {
        self.counters.get(&counter_id).map(|f| f.counter).unwrap_or(0)
    }

    pub fn increment_counter(&mut self, counter_id: u16, value: i64) {
        let current = self.get_counter(counter_id);
        self.set_counter(counter_id, current + value)
    }
}

