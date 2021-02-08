use tonic::Request;

use types::TokensReply;

use crate::storage::RedisRefreshTokenStorage;
use crate::token::JWTTokensService;

pub mod types {
    tonic::include_proto!("games.cheetah.cerberus.types");
}
pub mod internal {
    tonic::include_proto!("games.cheetah.cerberus.internal");
}

pub mod external {
    tonic::include_proto!("games.cheetah.cerberus.external");
}
