use rand::distr::Alphanumeric;
use rand::{thread_rng, Rng};
use crate::modules::authentication_error::AuthenticationError;

pub fn prepare_client_first_message(mechanism: &str) -> Result<Vec<u8>, AuthenticationError> {
    match mechanism {
        "SCRAM-SHA-256" => {
            let client_nonce = generate_nonce(18);
            let client_first_message = format!("n,,n={},r={}", "cypher_Dev", client_nonce);
            Ok(create_password_message(&client_first_message))
        },
        _ => Err(AuthenticationError::UnsupportedMechanism(mechanism.to_owned())),
    }
}

fn generate_nonce(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn create_password_message(client_nonce: &str) -> Vec<u8> {
    let mut message = Vec::new();

    let message_type = 'p' as u8;
    message.push(message_type); // p

    let mechanism = "SCRAM-SHA-256\0"; // sasl authentication mechanism

    let length = 4 + mechanism.len() + 4 + client_nonce.len();
    message.extend_from_slice(&(length as u32).to_be_bytes()); // total message length after 'p'

    message.extend_from_slice(mechanism.as_bytes()); // sasl authentication mechanism

    message.extend_from_slice(&(client_nonce.len() as u32).to_be_bytes()); // client first message length

    message.extend_from_slice(&client_nonce.as_bytes()); // client first message

    // ownership moved out
    message
}

// TODO: take protocol version as input
pub fn prepare_handshake_message(user: &str) -> Vec<u8> {
    let params = prepare_handshake_params(user);
    // calculate full length = 4 (length field itself) + 4 (protocol) + params.len()
    let len = 4 + 4 + params.len();
    let mut message = Vec::new();
    message.extend_from_slice(&(len as i32).to_be_bytes());       // length
    message.extend_from_slice(&(196608 as i32).to_be_bytes());    // protocol 3.0
    message.extend_from_slice(&params);

    message
}

fn prepare_handshake_params(user: &str) -> Vec<u8> {
    let mut params = Vec::new();
    params.extend_from_slice(b"user\0");
    params.extend_from_slice(user.as_bytes());
    params.push(0);
    params.extend_from_slice(b"replication\0");
    params.extend_from_slice(b"true\0");
    params.push(0); // terminator

    params
}