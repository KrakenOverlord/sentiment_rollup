mod database;

use database::Database;
use dotenv::dotenv;
use log::{error, info};

fn main() {
    dotenv().ok();
    env_logger::init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}