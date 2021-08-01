use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../../proto/matches/Factory/matches.factory.internal.proto",
                "../../../proto/matches/Relay/matches.relay.internal.proto",
                "../../../proto/matches/Matchmaking/matches.matchmaking.external.proto",
            ],
            &[
                "../../../proto/matches/Matchmaking/",
                "../../../proto/matches/Relay/",
                "../../../proto/matches/Factory/",
            ],
        )?;
    Result::Ok(())
}
