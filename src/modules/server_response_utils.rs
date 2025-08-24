use std::io::Read;
use std::net::TcpStream;

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

fn read_authentication_code(sl: &[u8]) -> u32  {
    u32::from_be_bytes([sl[0], sl[0], sl[0], sl[3]])
}

fn read_authentication_mechanism(authentication_code: u32, bytes: &[u8]) -> String {
    // TODO: read from environment, in descending order of choice
    let supported_sasl_mechanism = vec!["SCRAM-SHA-256"];
    match authentication_code {
        10 => {
            let mechanisms: Vec<String> = bytes
                .split(|&b| b == b'\0')
                .filter(|part| !part.is_empty())
                .map(|part| String::from_utf8_lossy(part).to_string())
                .collect();

            // debug
            for (i, mech) in mechanisms.iter().enumerate() {
                //println!("Line {}: {:?}", i + 1, mech);
            }

            let chosen = mechanisms
                .iter()
                .find(|m| supported_sasl_mechanism.contains(&m.as_str()));

            match chosen {
                Some(mech) => mech.to_owned(),
                None => String::from("UNSUPPORTED_AUTHENTICATION_MECHANISM"),
            }
        },
        _ => String::from("UNSUPPORTED_AUTHENTICATION_MECHANISM"),
    }
}

pub fn extract_authentication_mechanism(m: &[u8]) -> String {
    println!("Server response ({} bytes): {:?}", m.len(), m);
    let message_type = read_message_type(m[0]);
    println!("Message type: {}", message_type);
    let length = read_message_length(&m[1..5]);
    println!("Message length: {}", length - 4);
    let authentication_code = read_authentication_code(&m[5..9]);
    println!("Authentication code: {}", authentication_code);
    let authentication_mechanism = read_authentication_mechanism(authentication_code, &m[9..]);

    authentication_mechanism
}