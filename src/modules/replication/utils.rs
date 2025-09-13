use crate::modules::debug_utils::bytes_to_utfstring;
use crate::modules::replication::command_utils::start_replication_command;
use crate::modules::sasl::server_response_utils::decode;
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn replication(tcp_stream: &mut TcpStream)  {
    println!("Starting replication for");
    start_replication_step(tcp_stream);
}


fn start_replication_step(stream: &mut TcpStream) -> () {
    //let message = "START_REPLICATION SLOT scopes_slot LOGICAL 0/0 (proto_version '1', publication_names 'scopes_pub')";
    //stream.write_all(message.as_bytes()).expect("Failed to write to server");
    let publication_names = ["scopes_pub"];
    let command = start_replication_command("scopes_slot", "LOGICAL",
                                            "1", &publication_names);

    match stream.write_all(&command) {
        Ok(_) => {
            println!("Successfully send START_REPLICATION message");
            let message = decode(stream);
            match message {
                Ok(m) => {
                    println!("Server START_REPLICATION response ({} bytes): {:?}", m.len(), m);
                    println!("Server START_REPLICATION response as string: {}", bytes_to_utfstring(m.as_slice()).unwrap());
                },
                Err(e) => {
                    println!("Error while decoding START_REPLICATION response: {}", e);
                }
            }
        },
        Err(e) => println!("Error while sending START_REPLICATION message: {}", e)
    }


    let mut buffer = vec![0; 1024]; // Create a buffer to store received data
    let mut bytes_read = stream.read(&mut buffer).expect("Failed to read from stream");

    while bytes_read > 0 {
        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", received_data);
        bytes_read = stream.read(&mut buffer).expect("Failed to read from stream");
    }
}
