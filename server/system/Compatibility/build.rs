use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(false).compile(
		&["../../../proto/system/Compatibility/system.compatibility.external.proto"],
		&["../../../proto/system/Compatibility/"],
	)
}