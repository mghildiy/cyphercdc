use crate::modules::tcp::utils::get_tcp_connection;

pub fn replication(host: &str, port: u16, user: &str)  {
    let stream = match get_tcp_connection(host, port) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to get TCP connection to PostgreSQL server: {}", e);
            return ;
        }
    };
    println!("Starting replication for {}:{}", host, port);
}