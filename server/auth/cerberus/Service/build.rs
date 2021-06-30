use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../../../proto/auth/cerberus/auth.cerberus.internal.proto",
                "../../../../proto/auth/cerberus/auth.cerberus.external.proto",
            ],
            &["../../../../proto/auth/cerberus/"],
        )?;

    Result::Ok(())
}
