use dotenv::dotenv;
use std::env;
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
