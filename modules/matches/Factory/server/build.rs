use std::io::Error;

fn main() -> Result<(), Error> {
	println!(
		"cargo:rerun-if-changed=../proto/matches.factory.internal.proto"
	);
	println!("cargo:rerun-if-changed=../proto/matches.factory.admin.proto");
	println!(
		"cargo:rerun-if-changed=../proto/matches/Registry/matches.registry.internal.proto"
	);
	println!("cargo:rerun-if-changed=../proto/matches/Relay/matches.relay.internal.proto");

	tonic_build::configure().build_client(false).compile(
		&[
			"../proto/matches.factory.internal.proto",
			"../proto/matches.factory.admin.proto",
		],
		&[
			"../proto/",
			"../../Registry/proto/",
			"../../Relay/proto/",
		],
	)?;

	// сервер нужен в тестах
	tonic_build::configure().compile(
		&[
			"../../Registry/proto/matches.registry.internal.proto",
			"../../Relay/proto/matches.relay.internal.proto",
		],
		&[
			"../../Registry/proto/",
			"../../Relay/proto/",
		],
	)?;

	Ok(())
}
