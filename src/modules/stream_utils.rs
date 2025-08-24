use crate::modules::client_request_utils::{prepare_client_first_message, prepare_handshake_message};
use crate::modules::server_response_utils::{decode, extract_authentication_mechanism};
use std::io;
use std::io::Write;
use std::net::TcpStream;

pub fn send_startup_message_and_process_response(host: &str, port: u16, user: &str) {
    let mut stream = match connect_to_server(host, port) {
        Ok(strm) => strm,
        Err(e) => {
            eprintln!("Could not connect to PostgreSQL server: {}", e);
            return ;
        }
    };

    let handshake_message = prepare_handshake_message(user);
    match stream.write_all(&handshake_message) {
        Ok(_) => {
            println!("Handshake message sent successfully");
            let message = decode(&mut stream);
            match message {
                Ok(m) => {
                    let authentication_mechanism = extract_authentication_mechanism(&m);
                    println!("Authentication mechanism: {}", authentication_mechanism);
                    match prepare_client_first_message(&authentication_mechanism) {
                        Ok(authentication_message) => {
                            match stream.write_all(authentication_message.as_slice()) {
                                Ok(_) => {
                                    let auth_response = decode(&mut stream);
                                    match auth_response {
                                        Ok(m) => {
                                            print!("Server response ({} bytes): {:?}", m.len(), m);
                                        },
                                        Err(e) => eprintln!("Error while sending authentication details: {}", e),
                                    }
                                },
                                Err(e) => eprintln!("Could not send message: {}", e),
                            }
                        },
                        Err(e) => eprintln!("Failed to prepare client message: {}", e)
                    }
                },
                Err(e) => eprintln!("Failed to read server response: {}", e),
            }

        },
        Err(e) => eprintln!("Failed to send startup message: {}", e),
    }
}

fn connect_to_server(host: &str, port: u16) -> Result<TcpStream, io::Error> {
    let server_addr = format!("{}:{}", host, port);

    TcpStream::connect(server_addr)
}
