use sqlx::postgres::PgPool;


async fn check_table_exist(pool: &PgPool, table_name: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!("
SELECT EXISTS (
    SELECT FROM
        pg_tables
    WHERE
        schemaname = 'public' AND
        tablename  = $1
    )
", table_name).fetch_one(pool).await?;

    Ok(res.exists.unwrap())
}

async fn drop_table(pool: &PgPool, table_name: &str) -> Result<(), sqlx::Error> {
    let sql = "DROP TABLE IF EXISTS ".to_owned() + table_name;
    sqlx::query(&sql).execute(pool).await?;

    Ok(())
}

async fn create_schema(pool: &PgPool, table_name: &str) -> Result<(), sqlx::Error> {
    let sql_create_table = format!("
CREATE TABLE IF NOT EXISTS {0}
(
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY,
    date character(10) NOT NULL,
    name character(40) NOT NULL,
    is_off boolean NOT NULL,
    CONSTRAINT {0}_pkey PRIMARY KEY (id)
)
", table_name);
    let sql_create_index = format!("
CREATE UNIQUE INDEX IF NOT EXISTS {0}_ux_date ON {0} (date)
", table_name);

    let mut tx = pool.begin().await?;
    sqlx::query(&sql_create_table)
    .execute(insert_row&mut tx)
    .await?;
    sqlx::query(&sql_create_index)
    .execute(&mut tx)
    .await?;
    tx.commit().await?;

    Ok(())
}


pub async fn test_sqlx(pool: &PgPool) -> Result<(), sqlx::Error> {
    let table_name = "holiday";
    if check_table_exist(&pool, table_name).await? {
        println!("found '{}' table, deleting...", table_name);
        drop_table(pool, table_name).await?;
    } else {
        println!("'{}' table not found, creating...", table_name);
        create_schema(pool, table_name).await?;

        let days:Vec<Day> = sqlx::query_as!(Day, "
SELECT date, name, is_off FROM holiday WHERE date LIKE $1
", "2022-10%").fetch_all(pool).await?;

        for day in days {
            println!("name: {} date: {} is_off_day: {}", day.name, day.date, day.is_off);
        }
    }

    Ok(())
}


#[derive(Default, Debug, Clone, PartialEq)]
pub struct Day { date: String, name: String, is_off: bool }
