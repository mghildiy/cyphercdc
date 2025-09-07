extern crate core;

mod modules;
mod config;

use crate::config::CONFIG;
use modules::stream_utils;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    dotenv::dotenv().ok();
    start_sasl_authentication(&*CONFIG.db_host, CONFIG.db_port.parse().unwrap(), &*CONFIG.db_user);

    //Open a new TCP connection.
    //  Run the same SASL handshake as before.
    //Then send START_REPLICATION SLOT ... LOGICAL ....
}

fn start_sasl_authentication(host: &str, port: u16, user: &str) {
    stream_utils::sasl_authentication(host, port, user);
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
