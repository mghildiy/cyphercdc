use std::io::Read;
use std::net::TcpStream;
use base64::Engine;
use base64::engine::general_purpose;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::modules::sasl::authentication_error::AuthenticationError;
use crate::modules::sasl::authentication_error::AuthenticationError::{ClientKeyGenerationFailed, IllegalState};
use crate::modules::sasl::rsi::Rsi;

type HmacSha256 = Hmac<Sha256>;

pub fn decode(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut header = [0u8; 5];
    stream.read_exact(&mut header)?; // always read exactly 5 bytes

    let message_type = header[0];
    let length = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);

    // The length includes itself (4 bytes) but not the type byte.
    // So the full message size = 1 (type) + length
    let mut body = vec![0u8; (length as usize) - 4];
    stream.read_exact(&mut body)?; // read the rest

    // Rebuild full message: type + length + body
    let mut full_message = Vec::with_capacity(1 + 4 + body.len());
    full_message.push(message_type);
    full_message.extend_from_slice(&header[1..]); // already includes length
    full_message.extend_from_slice(&body);

    Ok(full_message)
}

fn read_message_type(by: u8) -> char {
    by as char
}

fn read_message_length(v: &[u8]) -> u32 {
    u32::from_be_bytes([v[0], v[1], v[2], v[3]])
}

fn read_authentication_type(sl: &[u8]) -> u32  {
    u32::from_be_bytes([sl[0], sl[0], sl[0], sl[3]])
}

fn payload_parts(payload: &[u8]) -> Vec<String> {
    let mechanisms: Vec<String> = payload
        .split(|&b| b == b'\0')
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).to_string())
        .collect();

    mechanisms
}

fn read_authentication_mechanism(authentication_code: u32, bytes: &[u8]) -> String {
    match authentication_code {
        10 => { //Authentication SASL
            let mechanisms = payload_parts(bytes);
            // TODO: read from environment, in descending order of choice
            let supported_sasl_mechanism = vec!["SCRAM-SHA-256"];
            let chosen = mechanisms
                .iter()
                .find(|m| supported_sasl_mechanism.contains(&m.as_str()));

            match chosen {
                Some(mech) => mech.to_owned(),
                None => String::from("UNSUPPORTED_AUTHENTICATION_MECHANISM"),
            }
        },
        11 => { // Authentication SASL continue
            let r_s_i = payload_parts(bytes);
            r_s_i.join(",").to_string()
        }
        _ => String::from("UNSUPPORTED_AUTHENTICATION_MECHANISM"),
    }
}

pub fn process_server_handshake_response(m: &[u8]) -> String {
    println!("Server handshake response ({} bytes): {:?}", m.len(), m);
    let message_type = read_message_type(m[0]);
    println!("Message type: {}", message_type);
    let length = read_message_length(&m[1..5]);
    println!("Message length: {}", length - 4);
    let authentication_type = read_authentication_type(&m[5..9]);
    println!("Authentication type: {}", authentication_type);
    let authentication_mechanism = read_authentication_mechanism(authentication_type, &m[9..]);
    println!("Authentication mechanism: {}", authentication_mechanism);

    authentication_mechanism
}

pub fn process_server_first_response(m: &[u8]) -> Rsi {
    println!("Server first response ({} bytes): {:?}", m.len(), m);
    let message_type = read_message_type(m[0]);
    println!("Message type: {}", message_type);
    let length = read_message_length(&m[1..5]);
    println!("Message length: {}", length - 4);
    let authentication_type = read_authentication_type(&m[5..9]);
    println!("Authentication type: {}", authentication_type);
    let r_s_i: Vec<String> = read_authentication_mechanism(authentication_type, &m[9..])
        .split(",")
        .map(|s| s.to_string())
        .collect();

    let  mut nonce = String::new();
    let mut salt = String::new();
    let mut iter_count: u32 = 0;
    for part in r_s_i {
        if let Some(value) = part.strip_prefix("r=") {
            nonce.push_str(value);
        } else if let Some(value) = part.strip_prefix("s=") {
            salt.push_str(value);
        } else if let Some(value) = part.strip_prefix("i=") {
            iter_count = value.parse::<u32>().unwrap();
        }
    }

    Rsi {
        nonce: nonce,
        salt: salt,
        iter_count: iter_count
    }
}

pub fn verify_server_signature(salted_password: &[u8], auth_message: &str, server_signature: &Vec<u8>)
    -> Result<(), AuthenticationError> {
    let mut hmac = HmacSha256::new_from_slice(salted_password)
        .map_err(|e| ClientKeyGenerationFailed(e.to_string()))?;
    hmac.update(b"Server Key");
    let server_key = hmac.finalize().into_bytes();

    let mut hmac2 = HmacSha256::new_from_slice(&server_key)
        .map_err(|e| ClientKeyGenerationFailed(e.to_string()))?;
    hmac2.update(auth_message.as_bytes());
    let expected_signature = hmac2.finalize().into_bytes();

    if server_signature == expected_signature.as_slice() {
        Ok(())
    } else {
        Err(IllegalState("Server signature mismatch".into()))
    }
}

pub fn extract_server_signature_bytes(full_message: &[u8]) -> Result<Vec<u8>, String> {
    // payload starts after type (1) + length (4)
    if full_message.len() <= 5 {
        return Err("message too short".into());
    }
    let payload = &full_message[5+4..];

    // find "v=" in payload (search raw bytes)
    let pos = payload.windows(2).position(|w| w == b"v=")
        .ok_or_else(|| "server final message: v= not found".to_string())?;

    let start = pos + 2; // start of base64 text

    // find end of base64: either comma, null, or end of payload
    let rel_end = payload[start..]
        .iter()
        .position(|&b| b == b',' || b == 0)
        .unwrap_or(payload[start..].len());
    let end = start + rel_end;

    let sig_b64_slice = &payload[start..end];

    // sig_b64_slice should be ASCII/base64 → convert to &str
    let sig_b64_str = std::str::from_utf8(sig_b64_slice)
        .map_err(|e| format!("server signature not valid UTF-8: {}", e))?;

    // decode base64 to raw signature bytes
    let sig_bytes = general_purpose::STANDARD
        .decode(sig_b64_str)
        .map_err(|e| format!("invalid base64 in server signature: {}", e))?;

    Ok(sig_bytes)
}
