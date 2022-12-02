pub mod matches {
	#![allow(
		clippy::empty_structs_with_brackets,
		clippy::derive_partial_eq_without_eq,
		clippy::wildcard_imports,
		clippy::return_self_not_must_use,
		clippy::clone_on_ref_ptr,
		clippy::similar_names
	)]

	pub mod realtime {
		pub mod shared {
			tonic::include_proto!("cheetah.matches.realtime.shared");
		}

		pub mod internal {
			tonic::include_proto!("cheetah.matches.realtime.internal");
		}
	}
}
