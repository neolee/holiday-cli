use sqlx::postgres::PgPoolOptions;


pub async fn test_sqlx(db_url: &str) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url).await?;

    let days:Vec<Day> = sqlx::query_as!(Day, "select date, name, is_off from holiday where date like $1", "2022-10%").fetch_all(&pool).await?;
    println!("data from db: {:?}", days);

    Ok(())
}


#[derive(Default, Debug, Clone, PartialEq)]
pub struct Day { date: String, name: String, is_off: bool }
