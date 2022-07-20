use std::io::Error;

fn main() -> Result<(), Error> {
	println!("cargo:rerun-if-changed=../proto/accounts.external.proto");
	tonic_build::configure()
		.build_server(true)
		.build_client(true)
		.compile(
			&["../proto/accounts.external.proto"],
			&["../proto/"],
		)
}
