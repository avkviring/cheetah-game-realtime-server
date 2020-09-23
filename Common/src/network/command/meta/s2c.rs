use crate::constants::ClientId;
use crate::network::command::{Decoder, Encoder};
use crate::network::command::meta::c2s::C2SMetaCommandInformation;
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Служебная информация для исходящей команды
///
#[derive(Debug, Clone,  PartialEq)]
pub struct S2CMetaCommandInformation {
    pub command_code: u8,

    ///
    /// Идентификатор клиента
    ///
    pub client: ClientId,

    ///
    /// Условное время создание команды на клиенте
    ///
    pub timestamp: u64,
}

impl S2CMetaCommandInformation {
    pub fn new(
        command_code: u8,
        client: ClientId,
        meta_from_client: &C2SMetaCommandInformation)
        -> Self {
        S2CMetaCommandInformation {
            command_code,
            client,
            timestamp: meta_from_client.timestamp,
        }
    }
}

impl Decoder for S2CMetaCommandInformation {
    fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
        Result::Ok(
            Self {
                command_code: buffer.read_u8()?,
                client: buffer.read_u16()?,
                timestamp: buffer.read_u64()?,
            }
        )
    }
}

impl Encoder for S2CMetaCommandInformation {
    fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
        buffer.write_u8(self.command_code)?;
        buffer.write_u16(self.client)?;
        buffer.write_u64(self.timestamp)?;
        Result::Ok(())
    }
}