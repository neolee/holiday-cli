use sqlx::postgres::PgPoolOptions;

pub mod data;
pub mod db;

use dotenv::dotenv;
use std::env;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db = env::var("DATABASE_URL").expect("wrong db url");
    let url_prefix = env::var("DATA_URL_PREFIX").expect("wrong data url");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db).await?;

    data::test_reqwest_serde(&url_prefix).await?;
    db::test_sqlx(&pool).await?;

    Ok(())
}
