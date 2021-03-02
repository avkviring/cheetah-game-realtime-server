use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../proto/cerberus.external.proto",
                "../../proto/cerberus.internal.proto",
            ],
            &["../../proto/"],
        )?;

    Result::Ok(())
}
