pub mod data;
pub mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use dotenv::dotenv;
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("wrong db url");

    data::test_reqwest_serde().await?;
    db::test_sqlx(&db_url).await?;

    Ok(())
}
