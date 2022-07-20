use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../../../../proto/matches/Relay/matches.relay.internal.proto");
	println!("cargo:rerun-if-changed=../../../../proto/matches/Relay/matches.relay.admin.proto");
	println!("cargo:rerun-if-changed=../../../../proto/matches/Relay/matches.relay.shared.proto");
	println!(
		"cargo:rerun-if-changed=../../../../proto/matches/Registry/matches.registry.internal.proto"
	);

	tonic_build::configure().compile(
		&[
			"../../../../proto/matches/Relay/matches.relay.internal.proto",
			"../../../../proto/matches/Relay/matches.relay.admin.proto",
		],
		&["../../../../proto/matches/Relay/"],
	)?;
	tonic_build::configure().build_server(false).compile(
		&["../../../../proto/matches/Registry/matches.registry.internal.proto"],
		&["../../../../proto/matches/Registry/"],
	)?;

	Ok(())
}
