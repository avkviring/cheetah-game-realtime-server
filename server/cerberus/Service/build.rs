use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../../proto/cerberus/cerberus.external.proto",
                "../../../proto/cerberus/cerberus.internal.proto",
            ],
            &["../../../proto/cerberus/"],
        )?;

    Result::Ok(())
}
