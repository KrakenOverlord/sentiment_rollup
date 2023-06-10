use anyhow::Result;
use chrono::{Utc, DateTime, NaiveDate};
use sqlx::{MySqlConnection, Connection};

// MySQL to Rust datatype mappings:
// https://docs.rs/sqlx-core/0.6.3/sqlx_core/mysql/types/index.html

#[derive(Debug)]
pub struct Event {
    pub id:         i32,
    pub event_id:   String,
    pub sentiment:  f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct Rollup {
    pub id:         i32,
    pub date:       NaiveDate,
    pub sentiment:  f32,
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
        let rows = sqlx::query_as!(Event, "SELECT * FROM events WHERE DATE(created_at) < CURDATE()")
            .fetch_all(&mut self.conn) 
            .await?;
        Ok(rows)
    }

    pub async fn get_rollup(&mut self, date: &str) -> Result<Option<Rollup>> {
        let res = sqlx::query_as!(Rollup, "SELECT * FROM rollups WHERE date = ?", date)
            .fetch_optional(&mut self.conn)
            .await?;

        Ok(res)
    }

    pub async fn insert_rollup(&mut self, date: &str, sentiment: f32) -> Result<()> {
        sqlx::query!(r#"INSERT INTO rollups (date, sentiment) VALUES (?, ?)"#, date, sentiment)
            .execute(&mut self.conn)
            .await?;

        Ok(())
    }

    pub async fn update_rollup(&mut self, date: &str, sentiment: f32) -> Result<()> {
        sqlx::query!(r#"UPDATE rollups SET sentiment = ? WHERE date = ?"#, sentiment, date)
            .execute(&mut self.conn)
            .await?;

        Ok(())
    }

    pub async fn delete_events(&mut self) -> Result<()> {
        let date = Utc::now().date_naive().to_string();
        sqlx::query!(r#"DELETE FROM events WHERE DATE(created_at) < ?"#, date)
            .execute(&mut self.conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test() {
        dotenv().ok();
        let mut database = Database::new().await.unwrap();
        let events = database.get_events().await.unwrap();

        for event in events {
            println!("{:#?}", event);
        }
    }

    #[tokio::test]
    async fn test2() {
        let f1 = 0.1;
        let f2 = 0.2;
        let sum = f1 + f2;
        println!("{}", sum);
    }
}