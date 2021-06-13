use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(
            &[
                "../../proto/cerberus/cerberus.external.proto",
                "../../proto/authentication/cookie.proto",
            ],
            &["../../proto/cerberus/", "../../proto/authentication/"],
        )?;

    Result::Ok(())
}
