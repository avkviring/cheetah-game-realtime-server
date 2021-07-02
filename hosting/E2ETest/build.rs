use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(
            &[
                "../../proto/auth/Cerberus/auth.cerberus.external.proto",
                "../../proto/auth/Cookie/auth.cookie.external.proto",
            ],
            &["../../proto/auth/Cerberus/", "../../proto/auth/Cookie/"],
        )?;

    Result::Ok(())
}
