use std::io::Error;

fn main() -> Result<(), Error> {
	std::env::set_var("PROTOC", protobuf_src::protoc());
	tonic_build::configure().compile(&["../../proto/internal.proto", "../../proto/admin.proto"], &["../../proto/"])?;
	tonic_build::configure()
		.build_server(false)
		.compile(&["../../proto/matches.registry.internal.proto"], &["../../proto/"])?;
	Ok(())
}
