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
		pub mod shared {
			tonic::include_proto!("cheetah.matches.relay.shared");
		}

		pub mod internal {
			tonic::include_proto!("cheetah.matches.relay.internal");
		}
	}

	pub mod registry {
		pub mod internal {
			tonic::include_proto!("cheetah.matches.registry.internal");
		}
	}
}
