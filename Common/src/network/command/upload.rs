use crate::constants::{GlobalObjectId, LocalObjectId};
use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::access::AccessGroups;
use crate::room::fields::GameObjectFields;

///
/// Загрузка объекта
/// - C -> S
///
#[derive(Debug)]
pub struct UploadGameObjectC2SCommand {
	pub local_id: LocalObjectId,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
}

///
/// Загрузка объекта
/// - S -> C
///
#[derive(Debug, Clone, PartialEq)]
pub struct UploadGameObjectS2CCommand {
	pub id: GlobalObjectId,
	pub fields: GameObjectFields,
}

impl CommandCode for UploadGameObjectS2CCommand {
	const COMMAND_CODE: u8 = 8;
}

impl CommandCode for UploadGameObjectC2SCommand {
	const COMMAND_CODE: u8 = 9;
}


impl Encoder for UploadGameObjectC2SCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u32(self.local_id)?;
		self.access_groups.encode(buffer)?;
		self.fields.encode(buffer)?;
		Result::Ok(())
	}
}

impl Decoder for UploadGameObjectC2SCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(UploadGameObjectC2SCommand {
			local_id: buffer.read_u32()?,
			access_groups: AccessGroups::decode(buffer)?,
			fields: GameObjectFields::decode(buffer)?,
		})
	}
}

impl Encoder for UploadGameObjectS2CCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u64(self.id)?;
		self.fields.encode(buffer)?;
		Result::Ok(())
	}
}

impl Decoder for UploadGameObjectS2CCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(UploadGameObjectS2CCommand {
			id: buffer.read_u64()?,
			fields: GameObjectFields::decode(buffer)?,
		})
	}
}