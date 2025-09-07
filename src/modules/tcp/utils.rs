use std::io;
use std::net::{Shutdown, TcpStream};

pub fn get_tcp_connection(host: &str, port: u16) -> Result<TcpStream, io::Error> {
    let server_addr = format!("{}:{}", host, port);

    TcpStream::connect(server_addr)
}

pub fn close_tcp_connection(stream: &TcpStream) -> Result<(), io::Error> {
    stream.shutdown(Shutdown::Both)
}