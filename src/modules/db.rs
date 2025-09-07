use std::env;
use postgres::{Client};
use postgres::NoTls;
use tokio_postgres::NoTls as TokioNoTls;
use crate::config::CONFIG;

pub fn connect_db () -> Result<Client, Box<dyn std::error::Error>> {
    println!("Connecting to database...");
    let conn_string = std::format!("host={} user={} password={} dbname={}", CONFIG.db_host,
                                   CONFIG.db_user, CONFIG.db_password, CONFIG.db_name);
    let conn = Client::connect(&conn_string, NoTls)?;

    return Ok(conn);
}

pub async fn logical_replication_connection() -> Result<tokio_postgres::Client, Box<dyn std::error::Error>> {
    println!("Consuming replication...");
    let conn_str = format!("host={} user={} password={} dbname={} replication=database",
    CONFIG.db_host, CONFIG.db_user, CONFIG.db_password, CONFIG.db_name);
    let (client, connection) = tokio_postgres::connect(&*conn_str, TokioNoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}