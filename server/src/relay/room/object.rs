use std::collections::HashMap;

type DataId = u32;


/// Игровой объект
/// содержит данные от пользователей
pub struct GameObject {
    pub(crate) id: u64,
    /// счетчики
    counters: HashMap<DataId, DataCounter>,
    /// структуры (для сервера это массивы данных)
    structs: HashMap<DataId, DataStruct>,
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
    pub fn new(owner: u16, local_id: u32, groups: Vec<u8>) -> GameObject {
        GameObject {
            id: (owner as u64).checked_shl(32).unwrap() + local_id as u64,
            counters: Default::default(),
            structs: Default::default(),
        }
    }
}

/// Хранение и управление списком игровых объектов
pub struct Objects {}

impl Objects {}

impl Default for Objects {
    fn default() -> Self {
        Objects {}
    }
}