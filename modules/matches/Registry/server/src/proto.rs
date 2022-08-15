pub mod matches {
	pub mod registry {
		pub mod internal {
			tonic::include_proto!("cheetah.matches.registry.internal");
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
}
