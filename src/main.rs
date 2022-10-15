use serde::{Deserialize,Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    use std::env;
    use dotenv::dotenv;
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("wrong db url");

    let res = reqwest::get("https://raw.githubusercontent.com/NateScarlet/holiday-cn/master/2022.json").await?;
    let data = res.json::<Root>().await?;
    let days = data.days;
    for day in days {
        println!("name: {} date: {} is_off_day: {}", day.name, day.date, day.is_off_day);
    }

    use sqlx::postgres::PgPoolOptions;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url).await?;

    let days:Vec<DBDay> = sqlx::query_as!(DBDay, "select date, name, is_off from holiday where date like $1", "2022-10%").fetch_all(&pool).await?;
    println!("data from db: {:?}", days);

    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq)]
struct DBDay { date: String, name: String, is_off: bool }


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "$id")]
    pub id: String,
    pub year: i64,
    pub papers: Vec<String>,
    pub days: Vec<Day>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub name: String,
    pub date: String,
    pub is_off_day: bool,
}
