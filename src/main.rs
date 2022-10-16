pub mod data;
pub mod db;

use dotenv::dotenv;
use std::env;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("wrong db url");
    let data_url_prefix = env::var("DATA_URL_PREFIX").expect("wrong data url");

    data::test_reqwest_serde(&data_url_prefix).await?;
    db::test_sqlx(&db_url).await?;

    Ok(())
}
