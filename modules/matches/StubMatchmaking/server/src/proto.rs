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
	pub mod realtime {
		pub mod shared {
			tonic::include_proto!("cheetah.matches.realtime.shared");
		}

		pub mod internal {
			tonic::include_proto!("cheetah.matches.realtime.internal");
		}
	}

	pub mod registry {
		pub mod internal {
			tonic::include_proto!("cheetah.matches.registry.internal");
		}
	}
}
