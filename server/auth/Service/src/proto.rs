pub mod auth {
    pub mod external {
        pub mod cookie {
            tonic::include_proto!("games.cheetah.auth.external.cookie");
        }
        pub mod google {
            tonic::include_proto!("games.cheetah.auth.external.google");
        }
    }
}
pub mod cerberus {
    pub mod types {
        tonic::include_proto!("games.cheetah.cerberus.types");
    }
    pub mod internal {
        tonic::include_proto!("games.cheetah.cerberus.internal");
    }
}
