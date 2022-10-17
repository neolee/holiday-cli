use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::PgPool;

pub mod data;
pub mod db;

use dotenv::dotenv;
use std::env;
use chrono::{Datelike, Utc};


async fn handle_data_of_year(url_prefix: &str, year: u32, pool: &PgPool, table_name: &str)
                             -> Result<(), Box<dyn std::error::Error>> {
    let days: Vec<data::Day> = data::get_holidays_of_year(&url_prefix, year).await?;

    print!("holidays in {}", year);
    for day in days {
        let data = db::Day {
            date: day.date,
            name: day.name,
            is_off: day.is_off_day
        };
        db::insert_row(pool, table_name, data).await?;
        print!(".");
    }
    println!("done.");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("loading config...");
    dotenv().ok();
    let db = env::var("DATABASE_URL").expect("wrong db url");
    let url_prefix = env::var("DATA_URL_PREFIX").expect("wrong data url");
    let table_name = env::var("TABLE_NAME").expect("wrong table name");
    println!("done.");

    print!("checking db schema...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db).await?;

    if db::check_table_exist(&pool, &table_name).await? {
        println!("'{}' table found.", table_name);
    } else {
        print!("creating '{}' table...", table_name);
        db::create_schema(&pool, &table_name).await?;
        println!("done.");
    }

    let mut args = Vec::<u32>::new();
    for arg in env::args().skip(1) {
        if let Ok(n) = arg.to_string().parse() {
            args.push(n)
        }
    }
    let begin_year = if args.len() >= 1 { args[0] } else { 2007 };
    let end_year = (if args.len() >= 2 { args[1] } else { Utc::now().year() as u32 }) + 1;

    println!("loading data from {} to {}...", begin_year, end_year);
    for year in begin_year..=end_year  {
        handle_data_of_year(&url_prefix, year, &pool, &table_name).await?;
    }
    println!("done.");

    Ok(())
}
