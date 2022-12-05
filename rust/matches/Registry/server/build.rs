use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../proto/matches.registry.internal.proto");
	println!("cargo:rerun-if-changed=../proto/matches.realtime.internal.proto");
	std::env::set_var("PROTOC", protobuf_src::protoc());
	tonic_build::configure()
		.build_client(false)
		.compile(&["../../Registry/proto/matches.registry.internal.proto"], &["../../Registry/proto/"])?;

	tonic_build::configure()
		.build_server(false)
		.compile(&["../../Realtime/proto/matches.realtime.internal.proto"], &["../../Realtime/proto/"])?;

	Ok(())
}
