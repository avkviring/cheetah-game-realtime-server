#[cfg(any(feature = "auth"))]
#[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
pub mod auth {
    pub mod cerberus {
        pub mod types {
            tonic::include_proto!("cheetah.auth.cerberus.types");
        }
        pub mod external {
            tonic::include_proto!("cheetah.auth.cerberus.external");
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
    pub mod user {
        pub mod internal {
            tonic::include_proto!("cheetah.auth.user.internal");
        }
    }
}

#[cfg(any(feature = "matches"))]
#[cfg_attr(docsrs, doc(cfg(feature = "matches")))]
pub mod matches {
    pub mod factory {
        pub mod internal {
            tonic::include_proto!("cheetah.matches.factory.internal");
        }
    }
    pub mod matchmaking {
        pub mod external {
            tonic::include_proto!("cheetah.matches.matchmaking.external");
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
