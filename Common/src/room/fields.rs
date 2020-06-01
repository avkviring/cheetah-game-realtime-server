use std::collections::HashMap;

use crate::constants::FieldID;
use crate::network::command::{Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Описание данных полей игрового объекта
///
#[derive(Debug, Clone, PartialEq)]
pub struct GameObjectFields {
    /// счетчики
    pub long_counters: HashMap<FieldID, i64>,
    pub float_counters: HashMap<FieldID, f64>,
    /// структуры (для сервера это массивы данных)
    pub structures: HashMap<FieldID, Vec<u8>>,
}


impl Default for GameObjectFields {
    fn default() -> Self {
        GameObjectFields {
            long_counters: Default::default(),
            float_counters: Default::default(),
            structures: Default::default(),
        }
    }
}

impl Encoder for GameObjectFields {
    fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
        write(buffer, &self.long_counters, |buffer, value| buffer.write_i64(*value))?;
        write(buffer, &self.float_counters, |buffer, value| buffer.write_f64(*value))?;
        write(buffer, &self.structures, |buffer, value| {
            buffer
                .write_u8(value.len() as u8)
                .and_then(|_| buffer.write_bytes(value))
        })?;
        Result::Ok(())
    }
}

impl Decoder for GameObjectFields {
    fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
        Result::Ok(GameObjectFields {
            long_counters: read(buffer, |buffer| buffer.read_i64())?,
            float_counters: read(buffer, |buffer| buffer.read_f64())?,
            structures: read(buffer, |buffer| {
                let size = buffer.read_u8()? as usize;
                buffer.read_to_vec(size)
            })?,
        })
    }
}

fn write<T>(buffer: &mut NioBuffer, value: &HashMap<FieldID, T>, writer: fn(&mut NioBuffer, value: &T) -> Result<(), NioBufferError>) -> Result<(), NioBufferError> {
    buffer
        .write_u16(value.len() as u16)
        .and_then(|_| {
            let result = value
                .iter()
                .map(|(key, value)| {
                    buffer
                        .write_u16(*key)
                        .and_then(|_| writer(buffer, value))
                })
                .find(|p| p.is_err());

            match result {
                None => { Result::Ok(()) }
                Some(e) => { e }
            }
        })
}

fn read<T>(buffer: &mut NioBuffer, element_reader: fn(&mut NioBuffer) -> Result<T, NioBufferError>) -> Result<HashMap<FieldID, T>, NioBufferError> {
    let mut result: HashMap<FieldID, T> = HashMap::new();
    for _ in 0..buffer.read_u16()? {
        let field = buffer.read_u16()?;
        let value = element_reader(buffer)?;
        result.insert(field, value);
    }
    Result::Ok(result)
}


