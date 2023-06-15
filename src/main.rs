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

// GET 'https://rest.coinapi.io/v1/ohlcv/BITSTAMP_SPOT_BTC_USD/history?period_id=1DAY&time_start=2023-06-13T00:00:00&limit=1' 
// Header "X-CoinAPI-Key: 9F4A5821-3F4F-4955-8FC2-2FEF6FDDE7F3" 
// [
//   {
//     "time_period_start": "2023-06-13T00:00:00.0000000Z",
//     "time_period_end": "2023-06-14T00:00:00.0000000Z",
//     "time_open": "2023-06-13T00:00:04.4060000Z",
//     "time_close": "2023-06-13T23:59:56.5310000Z",
//     "price_open": 25906,
//     "price_high": 26422,
//     "price_low": 25718,
//     "price_close": 25934,
//     "volume_traded": 1895.95998407,
//     "trades_count": 15847
//   }
// ]
async fn get_historical_bitcoin_price(date: &str) -> Result<i32> {
    let start_time = format!("{}T00:00:00", date);
    let query = format!("https://rest.coinapi.io/v1/ohlcv/BITSTAMP_SPOT_BTC_USD/history?period_id=1DAY&time_start={}&limit=1", start_time);

    let client = reqwest::Client::new();
    let res = client
        .get(query)
        .header("X-CoinAPI-Key", std::env::var("COINAPI_KEY")?)
        .send()
        .await?
        .text()
        .await?;

    let v: Value = serde_json::from_str(&res)?;
    let price = v[0]["price_close"].to_string().parse::<i32>().unwrap();

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
    // async fn test_coingecko_api() -> Result<()> {
    //     let days_ago = 5;
    //     let query = format!("https://api.coingecko.com/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days={}&interval=daily", days_ago);
    //     let res = reqwest::get(query)
    //         .await?
    //         .json::<HistoricalBitcoin>()
    //         .await?;

    //     let price = *res.prices.first().unwrap().last().unwrap() as i32;
    //     println!("Price = {}", price);

    //     Ok(())
    // }
}