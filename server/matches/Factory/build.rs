use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(true).compile(
		&[
			"../../../proto/matches/Factory/matches.factory.internal.proto",
			"../../../proto/matches/Registry/matches.registry.internal.proto",
			"../../../proto/matches/Relay/matches.relay.types.proto",
			"../../../proto/matches/Relay/matches.relay.internal.proto",
		],
		&[
			"../../../proto/matches/Factory/",
			"../../../proto/matches/Registry/",
			"../../../proto/matches/Relay/",
		],
	)?;
	Result::Ok(())
}
