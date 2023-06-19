mod database;

use anyhow::Result;
use database::{Database, Event};
use dotenv::dotenv;
use log::info;
use serde_json::{self, Value};
use std::collections::HashMap;

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
        let price = get_historical_bitcoin_price(date).await?;
        let rollup = database.get_rollup(date).await?;
        match rollup {
            Some(r) => {
                let next_value = r.sentiment + value;
                database.update_rollup(date, price, next_value).await?;
            },
            None => {
                database.insert_rollup(date, price, *value).await?;
            },
        }
    }

    // Delete all processed events
    let ids = events.iter().map(|v| v.id).collect();
    database.delete_events(&ids).await?;

    Ok(())
}

// This will return the closing price for the specified date.
// If the specified date is today, then the closing price is the current price.
// 
// GET 'https://rest.coinapi.io/v1/ohlcv/BITSTAMP_SPOT_BTC_USD/history?period_id=1DAY&time_start=2023-06-13T00:00:00&limit=1' 
// Header "X-CoinAPI-Key: [API KEY]" 
// [
//   {
    // "time_period_start": "2023-06-19T00:00:00.0000000Z",
    // "time_period_end": "2023-06-20T00:00:00.0000000Z",
    // "time_open": "2023-06-19T00:00:09.4530000Z",
    // "time_close": "2023-06-19T14:15:19.5430000Z",
    // "price_open": 26336,
    // "price_high": 26566,
    // "price_low": 26260,
    // "price_close": 26473,
    // "volume_traded": 585.99309704,
    // "trades_count": 5515
//   }
// ]
async fn get_historical_bitcoin_price(date: &str) -> Result<i32> {
    let query = format!("https://rest.coinapi.io/v1/ohlcv/BITSTAMP_SPOT_BTC_USD/history?period_id=1DAY&time_start={}&limit=1", date);

    let client = reqwest::Client::new();
    let res = client
        .get(query)
        .header("X-CoinAPI-Key", std::env::var("COINAPI_KEY")?)
        .send()
        .await?
        .text()
        .await?;

    let v: Value = serde_json::from_str(&res)?;
    let price = v[0]["price_close"].to_string().parse::<i32>()?;

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
        assert_eq!(price, 25934);

        let price = get_historical_bitcoin_price("2023-06-14").await?;
        assert_eq!(price, 25127);

        let price = get_historical_bitcoin_price("2023-06-15").await?;
        assert_eq!(price, 25051);
        Ok(())
    }

    // #[tokio::test]
    // async fn test() -> Result<()> {
    //     Ok(())
    // }
}