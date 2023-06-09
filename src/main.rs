mod database;

use anyhow::Result;
use chrono::NaiveDate;
use database::Database;
use dotenv::dotenv;
use log::{error, info};
use sqlx::types::{BigDecimal, time::Date};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut database = Database::new().await?;

    let rollups = get_rollups(&mut database).await?;
    for (date, value) in rollups.iter() {
        let rollup = database.get_rollup(date).await?;
        match rollup {
            Some(r) => {
                let next_value = r.sentiment + value;
                database.update_rollup(date, &next_value);
            },
            None => {
                database.insert_rollup(date, value);
            },
        }
    }

    Ok(())
}

async fn get_rollups(database: &mut Database) -> Result<HashMap<String, BigDecimal>> {
    let events = database.get_events().await?;
    let mut rollups: HashMap<String, BigDecimal> = HashMap::new();
    for event in events {
        // Create a key
        let date = event.created_at.date().to_string();
        println!("{}", date);

        // Get new sentiment value
        let current_value = rollups.get(&date);
        let next_value = match current_value {
            Some(v) => v + event.sentiment,
            None => event.sentiment,
        };

        rollups.insert(date, next_value);
    }
    return Ok(rollups);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}