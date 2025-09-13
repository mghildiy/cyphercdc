extern crate core;

mod modules;
mod config;
pub mod dto;

use crate::config::CONFIG;
use crate::modules::replication::utils as replication_utils;
use crate::modules::sasl::authentication_error::AuthenticationError;
use modules::sasl::utils;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    dotenv::dotenv().ok();
    // sasl authentication
    match start_sasl_authentication(&*CONFIG.db_host, CONFIG.db_port.parse().unwrap(), &*CONFIG.db_user
    ) {
        Ok(mut tcpstream) => {
            start_replication(&mut tcpstream)
        },
        Err(error) => eprintln!("{}", error),
    }
    // replication
}

fn start_sasl_authentication(host: &str, port: u16, user: &str) -> Result<TcpStream, AuthenticationError> {
    utils::sasl_authentication(host, port, user)
}

fn start_replication(tcp_stream: &mut TcpStream) {
    replication_utils::replication(tcp_stream);
}

fn start_replication_step(host: String, port: i32) -> () {
    let server_address = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(server_address).expect("Failed to connect to server");

    let message = "START_REPLICATION SLOT scopes_slot LOGICAL 0/0 (proto_version '1', publication_names 'scopes_pub')";
    stream.write_all(message.as_bytes()).expect("Failed to write to server");

    let mut buffer = vec![0; 1024]; // Create a buffer to store received data
    let mut bytes_read = stream.read(&mut buffer).expect("Failed to read from stream");

    while bytes_read > 0 {
        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", received_data);
        bytes_read = stream.read(&mut buffer).expect("Failed to read from stream");
    }
}

async fn process_replication(client: tokio_postgres::Client) -> () {
    println!("Processing replication....");

    let query = "START_REPLICATION SLOT scopes_slot LOGICAL 0/0 (proto_version '1', publication_names 'scopes_pub')";
    match client.simple_query(query).await {
        Ok(messages) => {
            for message in messages {
                match message {
                    tokio_postgres::SimpleQueryMessage::Row(row) => {
                        println!("Replication message: {:?}", row);
                    }
                    _ => {}
                }
            }
        }
        Err(e) => eprintln!("Failed to start replication: {}", e),
    }
}
