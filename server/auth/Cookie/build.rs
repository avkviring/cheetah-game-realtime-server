use std::io::Error;

fn main() -> Result<(), Error> {
	tonic_build::configure().build_server(true).build_client(true).compile(
		&[
			// auth
			"../../../proto/auth/Cerberus/auth.cerberus.internal.proto",
			"../../../proto/auth/Cerberus/auth.cerberus.types.proto",
			"../../../proto/auth/Cookie/auth.cookie.external.proto",
			"../../../proto/auth/User/auth.user.internal.proto",
		],
		&[
			"../../../proto/auth/Cerberus/",
			"../../../proto/auth/Cookie/",
			"../../../proto/auth/User/",
		],
	)
}
