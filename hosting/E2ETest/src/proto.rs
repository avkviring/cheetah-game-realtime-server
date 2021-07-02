pub mod auth {
    pub mod cerberus {
        pub mod types {
            tonic::include_proto!("cheetah.auth.cerberus.types");
        }

        pub mod external {
            tonic::include_proto!("cheetah.auth.cerberus.external");
        }
    }

    pub mod cookie {
        pub mod external {
            tonic::include_proto!("cheetah.auth.cookie.external");
        }
    }
}

