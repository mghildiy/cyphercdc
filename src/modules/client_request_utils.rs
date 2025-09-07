use base64::Engine;
use base64::engine::general_purpose;
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use rand::distr::Alphanumeric;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use crate::modules::authentication_error::AuthenticationError;
use crate::modules::authentication_error::AuthenticationError::{ClientKeyGenerationFailed, IllegalState};
use crate::modules::dto::ClientSecondMessage;
use crate::modules::rsi::Rsi;

type HmacSha256 = Hmac<Sha256>;
pub fn prepare_client_second_message(client_first_message: &[u8], mechanism: &str, rsi: &Rsi, password: &str)
    -> Result<ClientSecondMessage, AuthenticationError> {
    match extract_client_nonce_from_first_message(client_first_message) {
        Some(client_nonce) => {
            process_client_first_message(client_nonce, mechanism, rsi, password)
        },
        None => Err(IllegalState(String::from("Client nonce absent in  first message")))
    }
}

fn process_client_first_message(client_nonce: &str, mechanism: &str, rsi: &Rsi, password: &str)
    ->  Result<ClientSecondMessage, AuthenticationError> {
    if(!client_nonce.as_bytes().starts_with("n,,".as_ref())) {
        return Err(IllegalState(String::from("Client first message missing prefix n,,")));
    }
    match prepare_salted_password(mechanism, rsi, password) {
        Ok(salted_password) => {
            match HmacSha256::new_from_slice(salted_password.as_slice()) {
                Ok(mut hmac) => {
                    let client_key = compute_client_key(hmac);
                    let stored_key = Sha256::digest(&client_key);
                    let (auth_message, client_final_message_without_proof) =
                        compute_client_second_auth_message(client_nonce.as_bytes(), rsi);
                    let client_signature = compute_client_signature(&stored_key, auth_message.as_bytes())?;
                    let client_proof = compute_client_proof(&client_key, &client_signature);
                    let final_message = create_client_final_message(&client_final_message_without_proof, &client_proof);
                    let framed = build_password_message(&final_message);
                    Ok(ClientSecondMessage::new(salted_password, auth_message, framed))
                    //Ok(framed)
                },
                Err(e) => Err(ClientKeyGenerationFailed(e.to_string()))
            }
        },
        Err(e) => Err(e)
    }
}

fn build_password_message(payload: &str) -> Vec<u8> {
    let payload_bytes = payload.as_bytes();
    let mut msg = Vec::with_capacity(1 + 4 + payload_bytes.len());
    msg.push(b'p'); // message type

    // length = 4 (the length field itself) + payload.len()
    let len = (4 + payload_bytes.len()) as u32;
    msg.extend_from_slice(&len.to_be_bytes());

    msg.extend_from_slice(payload_bytes);
    msg
}

fn create_client_final_message(client_final_message_without_proof: &str, client_proof: &[u8]) -> String {
    let proof_b64 = base64::engine::general_purpose::STANDARD.encode(client_proof);
    format!("{},p={}", client_final_message_without_proof, proof_b64)
}

fn extract_client_nonce_from_first_message(message: &[u8]) -> Option<&str> {
    let mut pos = 1 + 4;
    while pos < message.len() && message[pos] != 0 {
        pos += 1;
    }
    pos += 1;
    pos += 4;

    std::str::from_utf8(&message[pos..]).ok()
}

fn compute_client_proof(client_key: &[u8], client_signature: &[u8]) -> Vec<u8> {
    assert_eq!(client_key.len(), 32);
    assert_eq!(client_signature.len(), 32);

    client_key.iter()
        .zip(client_signature.iter())
        .map(|(a, b)| a ^ b)
        .collect()
}

fn compute_client_second_auth_message(client_first_message: &[u8], rsi: &Rsi) -> (String, String) {
    let client_first_message_bare =
        client_first_message.strip_prefix(b"n,,").unwrap();

    let server_first_message =
        format!("r={},s={},i={}", rsi.nonce, rsi.salt, rsi.iter_count);

    let client_final_message_without_proof =
        format!("c=biws,r={}", rsi.nonce);

    let auth_message = format!(
        "{},{},{}",
        String::from_utf8_lossy(client_first_message_bare),
        server_first_message,
        client_final_message_without_proof
    );

    (auth_message, client_final_message_without_proof)
}


fn compute_client_signature(stored_key: &[u8], auth_message: &[u8]) -> Result<Vec<u8>, AuthenticationError> { {}
    match HmacSha256::new_from_slice(stored_key) {
            Ok(mut hmac) => {
                hmac.update(auth_message);
                let result = hmac.finalize().into_bytes();
                let mut signature = [0u8; 32];
                signature.copy_from_slice(&result);

                Ok(Vec::from(signature))
            },
            Err(e) => Err(ClientKeyGenerationFailed(e.to_string()))
    }
}

fn compute_client_key(mut hmac: HmacSha256) -> Vec<u8> {
    hmac.update(b"Client Key");
    let result = hmac.finalize();
    let bytes = result.into_bytes();

    bytes.to_vec()
}

pub fn prepare_salted_password(mechanism: &str, rsi: &Rsi, password: &str) -> Result<Vec<u8>, AuthenticationError> {
    match mechanism {
        "SCRAM-SHA-256" => {
            let salt = general_purpose::STANDARD.decode(&rsi.salt)
                .expect("Invalid base64 salt from server");
            // allocate space for the derived key
            let mut salted_password = [0u8; 32]; // SCRAM-SHA-256 always outputs 32 bytes
            pbkdf2_hmac::<Sha256>(
                password.as_bytes(),
                &salt,
                rsi.iter_count,
                &mut salted_password,
            );

            Ok(Vec::from(salted_password))
        },
        _ => Err(AuthenticationError::UnsupportedMechanism(mechanism.to_owned())),
    }
}

pub struct ClientFirstMessage {
    message: String,
    client_nonce: String
}

pub fn prepare_client_first_message(mechanism: &str) -> Result<Vec<u8>, AuthenticationError> {
    match mechanism {
        "SCRAM-SHA-256" => {
            let client_nonce = generate_nonce(18);
            let client_nonce = format!("n,,n={},r={}", "cypher_dev", client_nonce);
            Ok(create_password_message(&client_nonce))
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