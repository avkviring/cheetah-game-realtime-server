use std::io::Error;

fn main() -> Result<(), Error> {
	proto_auth()?;
	proto_matches()?;

	Ok(())
}

#[cfg(not(any(feature = "auth", doc)))]
fn proto_auth() -> Result<(), Error> {
	Ok(())
}

#[cfg(not(any(feature = "matches", doc)))]
fn proto_matches() -> Result<(), Error> {
	Ok(())
}

#[cfg(any(feature = "auth"))]
fn proto_auth() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(true).compile(
		&[
			// auth
			"../../proto/auth/Cerberus/auth.cerberus.internal.proto",
			"../../proto/auth/Cerberus/auth.cerberus.external.proto",
			"../../proto/auth/Cookie/auth.cookie.external.proto",
			"../../proto/auth/Google/auth.google.external.proto",
			"../../proto/auth/User/auth.user.internal.proto",
		],
		&[
			"../../proto/auth/Cerberus/",
			"../../proto/auth/Cookie/",
			"../../proto/auth/Google/",
			"../../proto/auth/User/",
		],
	)
}

#[cfg(any(feature = "matches"))]
fn proto_matches() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(true).compile(
		&[
			"../../proto/matches/Factory/matches.factory.internal.proto",
			"../../proto/matches/Matchmaking/matches.matchmaking.external.proto",
			"../../proto/matches/Registry/matches.registry.internal.proto",
			"../../proto/matches/Relay/matches.relay.types.proto",
			"../../proto/matches/Relay/matches.relay.internal.proto",
		],
		&[
			"../../proto/matches/Factory/",
			"../../proto/matches/Matchmaking/",
			"../../proto/matches/Registry/",
			"../../proto/matches/Relay/",
		],
	)
}
