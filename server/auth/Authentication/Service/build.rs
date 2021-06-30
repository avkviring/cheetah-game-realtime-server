use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../../../proto/auth/Cookie/auth.cookie.external.proto",
                "../../../../proto/auth/Google/auth.google.external.proto",
            ],
            &[
                "../../../../proto/auth/Cerberus/",
                "../../../../proto/auth/Cookie/",
                "../../../../proto/auth/Google/",
            ],
        )?;

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["../../../../proto/auth/Cerberus/auth.cerberus.internal.proto"],
            &["../../../../proto/auth/Cerberus/"],
        )?;

    Result::Ok(())
}
