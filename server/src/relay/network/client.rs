use std::net::TcpStream;

pub struct ClientStream {
    stream: Option<TcpStream>
}

impl ClientStream {
    pub fn stub() -> ClientStream {
        ClientStream {
            stream: Option::None
        }
    }
}
