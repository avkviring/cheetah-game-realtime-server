use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["../../proto/cookie.proto", "../../proto/google.proto"],
            &["../../../cerberus/proto/", "../../proto/"],
        )?;

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../../../cerberus/proto/cerberus.internal.proto"],
            &["../../../cerberus/proto/", "../../proto/"],
        )?;

    Result::Ok(())
}
