use std::net::TcpStream;

pub struct ClientStream {
    pub(crate) stream: Option<TcpStream>
}