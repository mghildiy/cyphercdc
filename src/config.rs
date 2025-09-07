use once_cell::sync::Lazy;
use crate::dto::DBConfig;

pub static CONFIG: Lazy<DBConfig> = Lazy::new(|| {
    dotenv::dotenv().ok();
    DBConfig::from_env()
});
