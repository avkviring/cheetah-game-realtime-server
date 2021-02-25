use std::io::Error;

fn main() -> Result<(), Error> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(&["../../proto/service.internal.proto"], &["../../proto/"])?;

    Result::Ok(())
}
