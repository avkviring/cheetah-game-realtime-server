pub mod matches {
	pub mod factory {
		pub mod internal {
			tonic::include_proto!("cheetah.matches.factory.internal");
		}
		pub mod admin {
			tonic::include_proto!("cheetah.matches.factory.admin");
		}
	}

	pub mod registry {
		pub mod internal {
			tonic::include_proto!("cheetah.matches.registry.internal");
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
}
