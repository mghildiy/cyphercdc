use postgres::{Client};
use postgres::NoTls;
use tokio_postgres::NoTls as TokioNoTls;

pub fn connect_db () -> Result<Client, Box<dyn std::error::Error>> {
    println!("Connecting to database...");

    let conn_string = String::from("host=localhost user=cypher_dev password=cypher_dev dbname=cypherdigitaltwin");
    let conn = Client::connect(&conn_string, NoTls)?;

    return Ok(conn);
}

pub async fn logical_replication_connection() -> Result<tokio_postgres::Client, Box<dyn std::error::Error>> {
    println!("Consuming replication...");

    let conn_str = "host=localhost user=cypher_dev password=cypher_dev dbname=cypherdigitaltwin replication=database";
    let (client, connection) = tokio_postgres::connect(conn_str, TokioNoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}