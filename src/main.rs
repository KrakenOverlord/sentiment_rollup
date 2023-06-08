mod database;

use database::Database;
use dotenv::dotenv;
use log::{error, info};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let database = Database::new();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}