use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure()
		.build_client(false)
		.build_server(true)
		.compile(
			&["../../proto/UserStore/userstore.external.proto"],
			&["../../proto/UserStore"],
		)
}
