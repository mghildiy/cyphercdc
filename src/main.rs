mod modules;

use std::io::{Read, Write};
use std::net::TcpStream;
use modules::db;
use modules::stream_utils;

fn main() {
    send_startup_message("localhost", 5432, "cypher_dev");
}

fn send_startup_message(host: &str, port: u16, user: &str) {
    let server_addr = format!("{}:{}", host, port);
    //let mut stream = TcpStream::connect(server_addr).expect("Could not connect to PostgreSQL");

    stream_utils::send_startup_message_and_process_response(host, port, user);
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

    /*if bytes_read > 0 {
        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", received_data);
    } else {
        print!("End of stream");
    }*/
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
