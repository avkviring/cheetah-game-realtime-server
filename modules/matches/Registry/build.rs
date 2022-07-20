use std::io::Error;

fn main() -> Result<(), Error> {
	println!(
		"cargo:rerun-if-changed=../../../proto/matches/Registry/matches.registry.internal.proto"
	);
	println!("cargo:rerun-if-changed=../../../proto/matches/Relay/matches.relay.internal.proto");
	tonic_build::configure().build_client(false).compile(
		&["../../../proto/matches/Registry/matches.registry.internal.proto"],
		&["../../../proto/matches/Registry/"],
	)?;

	tonic_build::configure().build_server(false).compile(
		&["../../../proto/matches/Relay/matches.relay.internal.proto"],
		&["../../../proto/matches/Relay/"],
	)?;

	Ok(())
}
