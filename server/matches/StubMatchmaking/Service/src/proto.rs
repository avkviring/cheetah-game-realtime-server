pub mod matches {
    pub mod matchmaking {
        pub mod external {
            tonic::include_proto!("cheetah.matches.matchmaking.external");
        }
    }
    pub mod factory {
        pub mod internal {
            tonic::include_proto!("cheetah.matches.factory.internal");
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
