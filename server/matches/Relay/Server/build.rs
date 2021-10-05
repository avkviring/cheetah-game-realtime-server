use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(true).compile(
		&[
			"../../../../proto/matches/Relay/matches.relay.internal.proto",
			"../../../../proto/matches/Relay/matches.relay.admin.proto",
		],
		&["../../../../proto/matches/Relay/"],
	)?;
	Result::Ok(())
}
