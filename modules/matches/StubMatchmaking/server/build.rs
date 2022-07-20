use std::io::Error;

fn main() -> Result<(), Error> {
	println!(
		"cargo:rerun-if-changed=../../Factory/proto/matches.factory.internal.proto"
	);
	println!("cargo:rerun-if-changed=../../Relay/proto/matches.relay.internal.proto");
	println!(
		"cargo:rerun-if-changed=../../Registry/proto/matches.registry.internal.proto"
	);
	println!("cargo:rerun-if-changed=../proto/matches.matchmaking.external.proto");

	tonic_build::configure().compile(
		&[
			"../../Factory/proto/matches.factory.internal.proto",
			"../../Relay/proto/matches.relay.internal.proto",
			"../../Registry/proto/matches.registry.internal.proto",
		],
		&[
			"../proto/",
			"../../Relay/proto/",
			"../../Factory/proto/",
			"../../Registry/proto/",
		],
	)?;

	tonic_build::configure().build_client(false).compile(
		&["../proto/matches.matchmaking.external.proto"],
		&["../proto/"],
	)?;
	Ok(())
}
