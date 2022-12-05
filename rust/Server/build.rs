use std::io::Error;

fn main() -> Result<(), Error> {
	std::env::set_var("PROTOC", protobuf_src::protoc());
	tonic_build::configure().compile(
		&[
			"../../proto/matches.realtime.internal.proto",
			"../../proto/matches.realtime.admin\
		.proto",
		],
		&["../../proto/"],
	)?;
	tonic_build::configure()
		.build_server(false)
		.compile(&["../../proto/matches.registry.internal.proto"], &["../../proto/"])?;
	Ok(())
}
