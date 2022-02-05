use std::io::Error;

fn main() -> Result<(), Error> {
	//
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
