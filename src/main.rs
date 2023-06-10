mod database;

use anyhow::Result;
use database::Database;
use dotenv::dotenv;
use log::info;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // Initialize database
    let mut database = Database::new().await?;

    // Get rollups
    let rollups = get_rollups(&mut database).await?;
    for (date, _) in rollups.iter() {
        info!("Processing rollup for {}", date);
    }

    // Create/update rollups in database
    for (date, value) in rollups.iter() {
        let rollup = database.get_rollup(date).await?;
        match rollup {
            Some(r) => {
                let next_value = r.sentiment + value;
                database.update_rollup(date, next_value).await?;
            },
            None => {
                database.insert_rollup(date, *value).await?;
            },
        }
    }

    // Delete all events prior to today
    database.delete_events().await?;

    Ok(())
}

async fn get_rollups(database: &mut Database) -> Result<HashMap<String, f32>> {
    let events = database.get_events().await?;
    info!("Found {} events.", events.len());

    let mut rollups: HashMap<String, f32> = HashMap::new();
    for event in events {
        // Create a key
        let date = event.created_at.date().to_string();

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
    use super::*;

    #[tokio::test]
    async fn test() -> Result<()> {
        dotenv().ok();

        let mut database = Database::new().await?;
        let rollups = get_rollups(&mut database).await?;
        for rollup in rollups {
            println!("{:#?}", rollup);
        }

        Ok(())
    }
}