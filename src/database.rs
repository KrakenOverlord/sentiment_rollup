use anyhow::Result;
use dotenv::dotenv;
use sqlx::{MySqlConnection, Connection, types::{BigDecimal, time::OffsetDateTime}};

pub struct Event {
    pub id:         i32,
    pub event_id:   String,
    pub sentiment:  BigDecimal,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct Database {
    conn: MySqlConnection,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let url = std::env::var("DATABASE_URL")?;
        let conn = MySqlConnection::connect(&url).await?;

        Ok(Database { conn })
    }

    pub async fn get_events(&mut self) -> Result<Vec<Event>> {
        let rows = sqlx::query_as!(Event, "SELECT * FROM events")
            .fetch_all(&mut self.conn) 
            .await?;

        Ok(rows)
    }

    // async fn record_to_maria(&self, pool: &Pool<MySql>, event: &SentimentEvent) -> Result<()> {
    //     let table_name = std::env::var("MARIADB_TABLE_NAME")?;
    //     let fields = "event_id, sentiment";
    //     let values = "?, ?";
    //     let query = format!("INSERT INTO {} ({}) VALUES ({})", table_name, fields, values);
    //     sqlx::query(&query)
    //         .bind(&event.id)
    //         .bind(&event.sentiment)
    //         .execute(pool)
    //         .await?;

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        dotenv().ok();
        let mut database = Database::new().await.unwrap();
        let events = database.get_events().await.unwrap();

        for event in events {
            println!("{}", event.event_id);
        }
    }
}