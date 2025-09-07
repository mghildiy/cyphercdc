use std::env;
use dotenv::dotenv;
use crate::modules::authentication_error::AuthenticationError;

pub struct ClientSecondMessage {
    salted_password: Vec<u8>,
    auth_message: String,
    password: Vec<u8>
}

impl ClientSecondMessage {
    pub(crate) fn new(p0: Vec<u8>, p1: String, p2: Vec<u8>) -> ClientSecondMessage {
        ClientSecondMessage {
            salted_password: p0,
            auth_message: p1,
            password: p2
        }
    }

    pub fn get_salted_password(&self) -> &Vec<u8> {
        &self.salted_password
    }

    pub fn get_auth_message(&self) -> &String {
        &self.auth_message
    }
    pub fn get_password(&self) -> &Vec<u8> {
        &self.password
    }

}

#[derive(Debug, Clone)]
pub struct DBConfig {
    pub db_host: String,
    pub db_port: String,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String
}

impl DBConfig {
    pub(crate) fn from_env() -> DBConfig {
        dotenv().ok(); // load .env into process env

        DBConfig {
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            db_port: env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()),
            db_name: env::var("DB_NAME").unwrap_or_else(|_| "mydb".to_string()),
            db_user: env::var("DB_USER").unwrap_or_else(|_| "myuser".to_string()),
            db_password: env::var("DB_PASSWORD").unwrap_or_else(|_| "mypassword".to_string()),
        }
    }
}
