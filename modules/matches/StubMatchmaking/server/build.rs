use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../../Factory/proto/matches.factory.internal.proto");
	println!("cargo:rerun-if-changed=../../Realtime/proto/matches.realtime.internal.proto");
	println!("cargo:rerun-if-changed=../../Registry/proto/matches.registry.internal.proto");
	println!("cargo:rerun-if-changed=../proto/matches.matchmaking.external.proto");

	tonic_build::configure().compile(
		&[
			"../../Factory/proto/matches.factory.internal.proto",
			"../../Realtime/proto/matches.realtime.internal.proto",
			"../../Registry/proto/matches.registry.internal.proto",
		],
		&[
			"../proto/",
			"../../Realtime/proto/",
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
