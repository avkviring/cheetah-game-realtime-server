use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_client(false).compile(
		&["../../Registry/proto/matches.registry.internal.proto"],
		&["../../Registry/proto/"],
	)?;
	Result::Ok(())
}
