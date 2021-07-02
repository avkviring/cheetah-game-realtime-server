use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../../../proto/matches/Registry/matches.registry.internal.proto"],
            &["../../../../proto/matches/Registry/"],
        )?;
    Result::Ok(())
}
