pub mod status {
    #![allow(
    clippy::empty_structs_with_brackets,
    clippy::derive_partial_eq_without_eq,
    clippy::wildcard_imports,
    clippy::return_self_not_must_use,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::must_use_candidate,
    clippy::doc_markdown
    )]
    tonic::include_proto!("cheetah.matches.realtime.status");
}
