use crate::modules::client_request_utils::{prepare_client_first_message, prepare_client_second_message, prepare_handshake_message};
use crate::modules::debug_utils::bytes_to_utfstring;
use crate::modules::server_response_utils::{decode, extract_server_signature_bytes, process_server_first_response, process_server_handshake_response, verify_server_signature};
use base64::Engine;
use std::fmt::Debug;
use std::{env, io};
use std::io::Write;
use std::net::TcpStream;

pub fn sasl_authentication(host: &str, port: u16, user: &str) {
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
                    let authentication_mechanism = process_server_handshake_response(&m);
                    match prepare_client_first_message(&authentication_mechanism) {
                        Ok(client_first_message) => {
                            println!("Client first message: {:?}", String::from_utf8_lossy(&*client_first_message));
                            match stream.write_all(client_first_message.as_slice()) {
                                Ok(_) => {
                                    let server_first_response = decode(&mut stream);
                                    match server_first_response {
                                        Ok(m) => {
                                            println!("Server first response ({} bytes): {:?}", m.len(), m);
                                            println!("Server first response as string: {}", bytes_to_utfstring(m.as_slice()).unwrap());
                                            let rsi = process_server_first_response(&m);
                                            println!("RSI: {}", rsi);
                                            // TODO read password from env
                                            let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string());
                                            match prepare_client_second_message(&client_first_message, &authentication_mechanism,
                                                                                &rsi, &password[..]) {
                                                Ok(client_second_message) => {
                                                    println!("Client second message as string: {}", String::from_utf8_lossy(&*client_second_message.get_password()));
                                                    match stream.write_all(client_second_message.get_password()) {
                                                        Ok(_) => {
                                                            let server_second_response = decode(&mut stream);
                                                            match server_second_response {
                                                                Ok(m) => {
                                                                    println!("Server second response: ({} bytes): {:?}", m.len(), m);
                                                                    println!("Server second response as string: {}", bytes_to_utfstring(m.as_slice()).unwrap());
                                                                    match extract_server_signature_bytes(m.as_slice()) {
                                                                        Ok(signature) => {
                                                                            match verify_server_signature(client_second_message.get_salted_password(),
                                                                                                    client_second_message.get_auth_message(),
                                                                                                    &signature) {
                                                                                Ok(_) => {
                                                                                    println!("Server signature valid");
                                                                                    drop(stream)
                                                                                },
                                                                                Err(e) => println!("Server signature invalid: {}", e),
                                                                            }
                                                                        },
                                                                        Err(e) => {}
                                                                    }
                                                                },
                                                                Err(e) => {}
                                                            }
                                                        },
                                                        Err(e) => eprintln!("Error while sending client first message: {}", e),
                                                    }
                                                },
                                                Err(e) => eprintln!("Could not prepare client second message response: {}", e),
                                            }
                                        },
                                        Err(e) => eprintln!("Error while sending client first message: {}", e),
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
