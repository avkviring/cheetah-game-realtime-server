use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(false).compile(
		&["../../../proto/liveops/ClientVersion/liveops.client.version.external.proto"],
		&["../../../proto/liveops/ClientVersion/"],
	)
}
