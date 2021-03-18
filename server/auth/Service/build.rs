use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../../proto/auth/cookie.proto",
                "../../../proto/auth/google.proto",
            ],
            &["../../../proto/cerberus/", "../../../proto/auth/"],
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
