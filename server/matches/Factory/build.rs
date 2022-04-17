use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_client(false).compile(
		&[
			"../../../proto/matches/Factory/matches.factory.internal.proto",
			"../../../proto/matches/Factory/matches.factory.admin.proto",
		],
		&[
			"../../../proto/matches/Factory/",
			"../../../proto/matches/Registry/",
			"../../../proto/matches/Relay/",
		],
	)?;

	// сервер нужен в тестах
	tonic_build::configure().compile(
		&[
			"../../../proto/matches/Registry/matches.registry.internal.proto",
			"../../../proto/matches/Relay/matches.relay.internal.proto",
		],
		&["../../../proto/matches/Registry/", "../../../proto/matches/Relay/"],
	)?;

	Result::Ok(())
}
