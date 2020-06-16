use crate::network::command::{CommandCode, Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};
use crate::room::access::AccessGroups;
use crate::room::fields::GameObjectFields;
use crate::room::object::ClientGameObjectId;

///
/// Загрузка объекта
///
#[derive(Debug, PartialEq, Clone)]
pub struct UploadGameObjectCommand {
	pub object_id: ClientGameObjectId,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
}


impl CommandCode for UploadGameObjectCommand {
	const COMMAND_CODE: u8 = 8;
}


impl Encoder for UploadGameObjectCommand {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		self.object_id.encode(buffer)?;
		self.access_groups.encode(buffer)?;
		self.fields.encode(buffer)?;
		Result::Ok(())
	}
}

impl Decoder for UploadGameObjectCommand {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		Result::Ok(UploadGameObjectCommand {
			object_id: ClientGameObjectId::decode(buffer)?,
			access_groups: AccessGroups::decode(buffer)?,
			fields: GameObjectFields::decode(buffer)?,
		})
	}
}