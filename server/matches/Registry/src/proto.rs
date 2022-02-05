pub mod matches {
	pub mod registry {
		pub mod internal {
			tonic::include_proto!("cheetah.matches.registry.internal");
		}
	}

	pub mod relay {
		// fixes error "::prost::Message could not find `shared` in `super`"
		pub mod shared {
			tonic::include_proto!("cheetah.matches.relay.shared");
		}

		pub mod internal {
			tonic::include_proto!("cheetah.matches.relay.internal");
		}
	}
}
