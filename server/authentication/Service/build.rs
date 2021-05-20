use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../../proto/authentication/cookie.proto",
                "../../../proto/authentication/google.proto",
            ],
            &["../../../proto/cerberus/", "../../../proto/authentication/"],
        )?;

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../../../proto/cerberus/cerberus.internal.proto"],
            &["../../../proto/cerberus/"],
        )?;

    Result::Ok(())
}
