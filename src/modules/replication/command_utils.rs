pub fn start_replication_command(slot_name: &str, replication_type: &str,
                                 proto_version: &str, publication_names: &[&str]) -> Vec<u8> {
    let mut command: Vec<u8> = Vec::new();
    command.push('Q' as u8);
    let payload = format!("START_REPLICATION SLOT {} {} 0/0 (proto_version '{}', \
        publication_names '{}')", slot_name, replication_type, proto_version, publication_names.join(","));
    println!("Payload: {}", payload);
    let payload_bytes = payload.as_bytes();
    //command.push("SLOT" as u8);
    let length: i32 = (4 + payload_bytes.len() + 1) as i32;
    let length_bytes = length.to_be_bytes();
    command.extend_from_slice(&length_bytes);
    command.extend_from_slice(&payload_bytes);
    command.push(0);

    command
}