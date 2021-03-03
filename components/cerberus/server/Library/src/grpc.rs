use tonic::metadata::MetadataMap;

use crate::grpc::AuthorizationError::*;
use crate::token::SessionTokenError;

#[derive(Debug)]
pub enum AuthorizationError {
    MissingHeader,
    WrongHeader,
    Token(SessionTokenError),
}

pub fn get_player_id(
    metadata: &MetadataMap,
    public_key: String,
) -> Result<u64, AuthorizationError> {
    match metadata.get("authorization") {
        None => Result::Err(MissingHeader),
        Some(value) => {
            let value = value.to_str().unwrap().to_string();
            let splitted: Vec<_> = value.split(" ").collect();
            if splitted.len() != 2 {
                Result::Err(WrongHeader)
            } else {
                let token = splitted.get(1).unwrap().to_string();
                let result = crate::token::JWTTokenParser::new(public_key).get_player_id(token);
                result.map_err(AuthorizationError::Token)
            }
        }
    }
}
