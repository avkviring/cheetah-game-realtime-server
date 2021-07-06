pub mod matches {
    pub mod factory {
        pub mod internal {
            tonic::include_proto!("cheetah.matches.factory.internal");
        }
    }

    pub mod registry {
        pub mod internal {
            tonic::include_proto!("cheetah.matches.registry.internal");
        }
    }

    pub mod relay {
        pub mod types {
            tonic::include_proto!("cheetah.matches.relay.types");
        }

        pub mod internal {
            tonic::include_proto!("cheetah.matches.relay.internal");
        }
    }
}
