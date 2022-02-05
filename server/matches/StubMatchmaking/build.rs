use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().compile(
		&[
			"../../../proto/matches/Factory/matches.factory.internal.proto",
			"../../../proto/matches/Relay/matches.relay.internal.proto",
			"../../../proto/matches/Registry/matches.registry.internal.proto",
		],
		&[
			"../../../proto/matches/Matchmaking/",
			"../../../proto/matches/Relay/",
			"../../../proto/matches/Factory/",
			"../../../proto/matches/Registry/",
		],
	)?;

	tonic_build::configure().build_client(false).compile(
		&[
			"../../../proto/matches/Matchmaking/matches.matchmaking.external.proto",
		],
		&[
			"../../../proto/matches/Matchmaking/",
		],
	)?;
	Result::Ok(())
}
