use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../proto/matches.realtime.internal.proto");
	println!("cargo:rerun-if-changed=../proto/matches.realtime.admin.proto");
	println!("cargo:rerun-if-changed=../proto/matches.realtime.shared.proto");
	println!("cargo:rerun-if-changed=../Registry/proto/matches.registry.internal.proto");

	tonic_build::configure().compile(
		&[
			"../proto/matches.realtime.internal.proto",
			"../proto/matches.realtime.admin.proto",
		],
		&["../proto/"],
	)?;
	tonic_build::configure().build_server(false).compile(
		&["../../Registry/proto/matches.registry.internal.proto"],
		&["../../Registry/proto/"],
	)?;

	Ok(())
}
