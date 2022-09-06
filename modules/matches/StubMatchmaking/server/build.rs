use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../proto/matches.factory.internal.proto");
	println!("cargo:rerun-if-changed=../proto/matches.realtime.internal.proto");
	println!("cargo:rerun-if-changed=../proto/matches.registry.internal.proto");
	println!("cargo:rerun-if-changed=../proto/matches.matchmaking.external.proto");

	tonic_build::configure().compile(
		&[
			"../proto/matches.factory.internal.proto",
			"../proto/matches.realtime.internal.proto",
			"../proto/matches.registry.internal.proto",
		],
		&["../proto/"],
	)?;

	tonic_build::configure()
		.build_client(false)
		.compile(&["../proto/matches.matchmaking.external.proto"], &["../proto/"])?;
	Ok(())
}
