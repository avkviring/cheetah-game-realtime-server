pub mod cerberus {
    pub mod types {
        tonic::include_proto!("games.cheetah.cerberus.types");
    }

    pub mod external {
        tonic::include_proto!("games.cheetah.cerberus.external");
    }
}

pub mod authentication {
    pub mod external {
        pub mod cookie {
            tonic::include_proto!("games.cheetah.authentication.external.cookie");
        }
    }
}
