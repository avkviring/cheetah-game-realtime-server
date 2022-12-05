use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../../../proto/matches.realtime.internal.proto");

	println!("cargo:rerun-if-changed=../../../proto/matches.realtime.shared.proto");

	std::env::set_var("PROTOC", protobuf_src::protoc());
	tonic_build::configure().compile(
		&[
			"../../../proto/matches.realtime.internal.proto",
			"../../../proto/matches.realtime.shared.proto",
		],
		&["../../../proto/"],
	)?;
	Ok(())
}
