use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &[
                "../../../../proto/match/Factory/internal.proto",
                "../../../../proto/match/Relay/types.proto",
            ],
            &[
                "../../../../proto/match/Factory/",
                "../../../../proto/match/Relay/",
            ],
        )?;
    Result::Ok(())
}
