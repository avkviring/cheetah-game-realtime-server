use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &[
                "../../../../proto/matches/Factory/matches.factory.internal.proto",
                "../../../../proto/matches/Relay/matches.relay.types.proto",
            ],
            &[
                "../../../../proto/matches/Factory/",
                "../../../../proto/matches/Relay/",
            ],
        )?;
    Result::Ok(())
}
