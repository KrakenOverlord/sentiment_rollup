mod database;

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use database::{Database, Event};
use dotenv::dotenv;
use log::{info, debug};
use serde::Deserialize;
use std::collections::HashMap;

// Response structs for: GET https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd
// {
//     "bitcoin": {
//         "usd": 25818
//     }
// }
// #[derive(Deserialize)]
// struct Bitcoin {
//     bitcoin: Price,
// }

// #[derive(Deserialize)]
// struct Price {
//     usd: i32,
// }

// Response structs for: GET api.coingecko.com/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days=1&interval=daily
// {
//     "prices": [
//       [
//         1686700800000,
//         25872.20645879509
//       ],
//       [
//         1686755836000,
//         25992.30353210715
//       ]
//     ]
// }
#[derive(Deserialize)]
struct HistoricalBitcoin {
    prices: Vec<Vec<f32>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // Initialize database
    let mut database = Database::new().await?;

    // Get all events
    let events = database.get_events().await?;
    info!("Found {} events.", events.len());
    if events.len() == 0 {
        return Ok(());
    }

    // Get rollups
    let rollups = get_rollups(&events).await?;
    for rollup in rollups.iter() {
        info!("{:?}", rollup);
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
                let price = get_historical_bitcoin_price(date).await?;
                database.insert_rollup(date, price, *value).await?;
            },
        }
    }

    // Delete all processed events
    let ids = events.iter().map(|v| v.id).collect();
    database.delete_events(&ids).await?;

    Ok(())
}

// GET https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd
// {"bitcoin":{"usd":25818}}
// async fn get_bitcoin_price() -> Result<i32> {
//     let res = reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
//         .await?
//         .json::<Bitcoin>()
//         .await?;

//     Ok(res.bitcoin.usd)
// }

// GET api.coingecko.com/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days=1&interval=daily
// Returns prices for every day since the date you specify
// {
//     "prices": [
//       [
//         1686787724000,
//         25083.61354357484
//       ]
//     ]
// }
async fn get_historical_bitcoin_price(date: &str) -> Result<i32> {
    let naive_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
    let current_date = Utc::now().date_naive();
    let days_ago = (current_date - naive_date).num_days();

    let query = format!("https://api.coingecko.com/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days={}&interval=daily", days_ago);
    let res = reqwest::get(query)
        .await?
        .json::<HistoricalBitcoin>()
        .await?;

    let price = *res.prices.first().unwrap().last().unwrap() as i32;
    debug!("Price was ${} on {}", price, date);
    Ok(price)
}

// Returns a HashMap with all events grouped by date with summed sentiment
async fn get_rollups(events: &Vec<Event>) -> Result<HashMap<String, f32>> {
    let mut rollups: HashMap<String, f32> = HashMap::new();
    for event in events {
        // Create a key
        let date = event.created_at.date_naive().to_string();

        // Calculate sentiment value
        let current_value = rollups.get(&date);
        let next_value = match current_value {
            Some(v) => v + event.sentiment,
            None => event.sentiment,
        };

        // Add key/value to map
        rollups.insert(date, next_value);
    }
    return Ok(rollups);
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    // INSERT INTO `events` (event_id, sentiment, created_at)
    // VALUES
    //     (1, -0.01, '2023-06-10 00:00:28'),
    //     (2, -0.1, '2023-06-10 00:01:39'),
    //     (3, 0.22, '2023-06-12 00:02:00'),
    //     (4, -0.2, '2023-06-14 00:02:00'),
    //     (5, 0.53, '2023-06-14 00:02:06');
    #[tokio::test]
    async fn test() -> Result<()> {
        dotenv().ok();

        let price = get_historical_bitcoin_price("2023-06-13").await?;
        assert_eq!(price, 25872);

        let price = get_historical_bitcoin_price("2023-06-14").await?;
        assert_eq!(price, 25107);

        let price = get_historical_bitcoin_price("2023-06-15").await?;
        assert_eq!(price, 25107);
        Ok(())
    }

    #[tokio::test]
    async fn test_coingecko_api() -> Result<()> {
        let days_ago = 5;
        let query = format!("https://api.coingecko.com/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days={}&interval=daily", days_ago);
        let res = reqwest::get(query)
            .await?
            .json::<HistoricalBitcoin>()
            .await?;

        let price = *res.prices.first().unwrap().last().unwrap() as i32;
        println!("Price = {}", price);

        Ok(())
    }
}