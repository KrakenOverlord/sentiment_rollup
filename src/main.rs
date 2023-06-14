mod database;

use anyhow::Result;
use database::Database;
use dotenv::dotenv;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Price {
    usd: i32,
}

// {"bitcoin":{"usd":25818}}
#[derive(Deserialize)]
struct Bitcoin {
    bitcoin: Price,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // Initialize database
    let mut database = Database::new().await?;

    // Get bitcoin prices

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

// GET https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd
// {"bitcoin":{"usd":25818}}
async fn get_bitcoin_prices() -> Result<Vec<i32>> {
    let body = reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .await?
        .json()
        .await?;

    Ok(vec![])
}

// Returns a HashMap for every day prior to today (UTC) with corresponding sentiment totals
async fn get_rollups(database: &mut Database) -> Result<HashMap<String, f32>> {
    let events = database.get_events().await?;
    info!("Found {} events.", events.len());

    let mut rollups: HashMap<String, f32> = HashMap::new();
    for event in events {
        // Create a key
        let date = event.created_at.date_naive().to_string();

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

    #[tokio::test]
    async fn test2() -> Result<()> {
        let res = reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
            .await?
            .json::<Bitcoin>()
            .await?;

        println!("{:?}", res.bitcoin.usd);

        Ok(())
    }
}