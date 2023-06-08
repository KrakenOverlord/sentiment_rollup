use anyhow::Result;
use chrono::Utc;
use sqlx::{MySql, mysql::MySqlPoolOptions, Pool};

#[derive(Debug)]
pub struct Database {
}

impl Database {
    // pub async fn new() -> Result<Self> {
    //     let url = std::env::var("MARIA_URL")?;
    //     let pool = MySqlPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&url).await?;

    //     Ok(Database { database: DatabaseType::MariaDB(pool) })
    // }

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
