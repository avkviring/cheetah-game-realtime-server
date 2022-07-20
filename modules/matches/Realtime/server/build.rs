use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../proto/matches.relay.internal.proto");
	println!("cargo:rerun-if-changed=../proto/matches.relay.admin.proto");
	println!("cargo:rerun-if-changed=../proto/matches.relay.shared.proto");
	println!(
		"cargo:rerun-if-changed=../Registry/proto/matches.registry.internal.proto"
	);

	tonic_build::configure().compile(
		&[
			"../proto/matches.relay.internal.proto",
			"../proto/matches.relay.admin.proto",
		],
		&["../proto/"],
	)?;
	tonic_build::configure().build_server(false).compile(
		&["../../Registry/proto/matches.registry.internal.proto"],
		&["../../Registry/proto/"],
	)?;

	Ok(())
}
