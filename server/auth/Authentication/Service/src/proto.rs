pub mod auth {
    pub mod cerberus {
        pub mod types {
            tonic::include_proto!("cheetah.auth.cerberus.types");
        }
        pub mod internal {
            tonic::include_proto!("cheetah.auth.cerberus.internal");
        }
    }
    pub mod cookie {
        pub mod external {
            tonic::include_proto!("cheetah.auth.cookie.external");
        }
    }
    pub mod google {
        pub mod external {
            tonic::include_proto!("cheetah.auth.google.external");
        }
    }
}
