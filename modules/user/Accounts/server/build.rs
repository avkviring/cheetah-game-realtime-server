use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../proto/user.accounts.external.proto");
	tonic_build::configure()
		.build_server(true)
		.build_client(true)
		.compile(&["../proto/user.accounts.external.proto"], &["../proto/"])
}
