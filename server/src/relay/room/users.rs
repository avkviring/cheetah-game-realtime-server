use crate::relay::room::structs::UserAuth;

impl UserAuth {
    pub(crate) fn new(hash: &str) -> UserAuth {
        UserAuth {
            hash: hash.to_string()
        }
    }
}