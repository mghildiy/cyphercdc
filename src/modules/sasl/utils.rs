use crate::modules::debug_utils::bytes_to_utfstring;
use crate::modules::sasl::client_request_utils::{prepare_client_first_message, prepare_client_second_message, prepare_handshake_message};
use crate::modules::sasl::server_response_utils::{decode, extract_server_signature_bytes, process_server_first_response, process_server_handshake_response, verify_server_signature};
use crate::modules::tcp::utils::get_tcp_connection;
use base64::Engine;
use std::fmt::Debug;
use std::io::Write;
use std::env;
use std::net::TcpStream;
use crate::modules::sasl::authentication_error::AuthenticationError;
use crate::modules::sasl::authentication_error::AuthenticationError::{ConnectionFailed, GenericError, IllegalState, SASLAuthenticationFailed};

pub fn sasl_authentication(host: &str, port: u16, user: &str) -> Result<TcpStream, AuthenticationError> {
    let mut stream = match get_tcp_connection(host, port) { //connect_to_server(host, port) {
        Ok(strm) => strm,
        Err(e) => {
            eprintln!("Could not connect to PostgreSQL server: {}", e);
            return Err(ConnectionFailed(format!("Connection failed for user: {}, address: {}:{}", user, host, port)))
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
                                                                                    //drop(stream)
                                                                                    //close_tcp_connection(&stream);
                                                                                    Ok(stream)
                                                                                },
                                                                                Err(e) => {
                                                                                    println!("Server signature invalid: {}", e);
                                                                                    Err(SASLAuthenticationFailed(format!("Server signature invalid for user: {}", user)))
                                                                                },
                                                                            }
                                                                        },
                                                                        Err(e) => {
                                                                            eprintln!("Server signature invalid: {}", e);
                                                                            Err(IllegalState(format!("Error while extracting server signature from server \
                                                                            second response for user: {}", user)))
                                                                        }
                                                                    }
                                                                },
                                                                Err(e) => {
                                                                    eprintln!("Error while decoding server second response: {}", e);
                                                                    Err(IllegalState(format!("Error while decoding server second response for user {}", user)))
                                                                }
                                                            }
                                                        },
                                                        Err(e) => {
                                                            eprintln!("Error while sending client second message: {}", e);
                                                            Err(ConnectionFailed(format!("Error while sending client second message for user: {}", user)))
                                                        },
                                                    }
                                                },
                                                Err(e) => {
                                                    eprintln!("Could not prepare client second message response: {}", e);
                                                    Err(GenericError(format!("Error while preparing client second message for user: {}", user)))
                                                },
                                            }
                                        },
                                        Err(e) => {
                                            eprintln!("Error while decoding client first message: {}", e);
                                            Err(IllegalState(format!("Error while decoding server first response for user {}", user)))
                                        },
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Could not send message: {}", e);
                                    Err(ConnectionFailed(format!("Error while sending client first message for user: {}", user)))
                                },
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to prepare client first message: {}", e);
                            Err(GenericError(format!("Error while preparing client first message for user: {}", user)))
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error while decoding handshake response: {}", e);
                    Err(IllegalState(format!("Error while decoding handshake response for user {}", user)))
                },
            }

        },
        Err(e) => {
            eprintln!("Failed to send handshake message: {}", e);
            Err(ConnectionFailed(format!("Error while sending handshake message for user: {}", user)))
        },
    }
}

/*fn connect_to_server(host: &str, port: u16) -> Result<TcpStream, io::Error> {
    let server_addr = format!("{}:{}", host, port);

    TcpStream::connect(server_addr)
}*/
