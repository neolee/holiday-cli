use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;

pub mod data;
pub mod db;

use chrono::{Datelike, Utc};
use dotenv::dotenv;
use std::env;

async fn load_data_of_year(
    url_prefix: &str,
    year: u32,
    pool: &PgPool,
    table_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let days: Vec<data::Day> = data::get_holidays_of_year(url_prefix, year).await?;

    print!("[{}]", year);
    for day in &days {
        db::insert_row(pool, table_name, &day.date, &day.name, day.is_off_day).await?;
        print!(".");
    }
    println!();

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
    let pool = PgPoolOptions::new().max_connections(5).connect(&db).await?;

    if db::check_table_exist(&pool, &table_name).await? {
        println!("'{}' table found.", table_name);
    } else {
        print!("creating '{}' table...", table_name);
        db::create_table(&pool, &table_name).await?;
        println!("done.");
    }

    let mut args = Vec::<u32>::new();
    for arg in env::args().skip(1) {
        if let Ok(n) = arg.to_string().parse() {
            args.push(n)
        }
    }

    if args == [0] {
        print!("dropping '{}' table...", table_name);
        db::drop_table(&pool, &table_name).await?;
        println!("done. (re-run this tool to start over)");

        return Ok(());
    }

    let begin_year = if let Some(&y) = args.first() { y } else { 2007 };
    let end_year = (if let Some(&y) = args.get(1) {
        y
    } else {
        Utc::now().year() as u32
    }) + 1;

    println!(
        "loading holiday data from year {} to {}",
        begin_year, end_year
    );
    for year in begin_year..=end_year {
        load_data_of_year(&url_prefix, year, &pool, &table_name).await?;
    }
    println!("done.");

    Ok(())
}
