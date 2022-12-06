use std::io::Error;

fn main() -> Result<(), Error> {
	std::env::set_var("PROTOC", protobuf_src::protoc());
	tonic_build::configure().compile(&["../../proto/internal.proto", "../../proto/shared.proto"], &["../../proto/"])?;
	Ok(())
}
