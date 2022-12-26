use std::io::Error;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
	#[cfg(target_os = "macos")]
	std::env::set_var("PROTOC", PathBuf::from("../../scripts/bin/mac/protoc"));
	#[cfg(target_os = "linux")]
	std::env::set_var("PROTOC", PathBuf::from("../../scripts/bin/lin/protoc"));
	#[cfg(target_os = "windows")]
	std::env::set_var("PROTOC", PathBuf::from("../../scripts/bin/win/protoc"));

	tonic_build::configure().build_client(false).compile(&["../../proto/registry.internal.proto"], &["../../proto/"])?;

	tonic_build::configure().build_server(false).compile(&["../../proto/internal.proto"], &["../../proto/"])?;

	Ok(())
}
