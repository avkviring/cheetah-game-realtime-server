use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(true).compile(
		&[
			"../../../proto/auth/Cerberus/auth.cerberus.internal.proto",
			"../../../proto/auth/Cerberus/auth.cerberus.types.proto",
			"../../../proto/auth/User/auth.user.internal.proto",
		],
		&["../../../proto/auth/Cerberus/", "../../../proto/auth/User/"],
	)
}
