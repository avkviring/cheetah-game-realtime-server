use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["../../../../proto/match/Relay/internal.proto"],
                 &["../../../../proto/match/Relay/"])?;
    Result::Ok(())
}
