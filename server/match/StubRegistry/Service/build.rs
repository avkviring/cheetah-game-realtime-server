use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../../../proto/match/Registry/internal.proto"],
            &["../../../../proto/match/Registry/"],
        )?;
    Result::Ok(())
}
