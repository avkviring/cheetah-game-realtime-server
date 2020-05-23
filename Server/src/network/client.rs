use std::net::TcpStream;

pub struct ClientStream {
    pub stream: Option<TcpStream>
}