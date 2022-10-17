use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::PgPool;

pub mod data;
pub mod db;

use dotenv::dotenv;
use std::env;


async fn handle_data_of_year(url_prefix: &str, year: u32, pool: &PgPool, table_name: &str)
                             -> Result<(), Box<dyn std::error::Error>> {
    let days: Vec<data::Day> = data::get_holidays_of_year(&url_prefix, year).await?;

    for day in days {
        let data = db::Day {
            date: day.date,
            name: day.name,
            is_off: day.is_off_day
        };
        db::insert_row(pool, table_name, data).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db = env::var("DATABASE_URL").expect("wrong db url");
    let url_prefix = env::var("DATA_URL_PREFIX").expect("wrong data url");
    let table_name = env::var("TABLE_NAME").expect("wrong table name");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db).await?;

    if db::check_table_exist(&pool, &table_name).await? {
        println!("found '{}' table", table_name);
    } else {
        println!("creating '{}' table", table_name);
        db::create_schema(&pool, &table_name).await?;
    }

    handle_data_of_year(&url_prefix, 2022, &pool, &table_name).await?;

    Ok(())
}
