use sqlx::postgres::PgPool;


pub async fn check_table_exist(pool: &PgPool, table_name: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!("
SELECT EXISTS (
    SELECT * FROM
        information_schema.tables
    WHERE
        table_type LIKE 'BASE TABLE' AND
        table_name = $1
    )
", table_name).fetch_one(pool).await?;

    Ok(res.exists.unwrap())
}

pub async fn drop_schema(pool: &PgPool, table_name: &str) -> Result<(), sqlx::Error> {
    let sql = "DROP TABLE IF EXISTS ".to_owned() + table_name;
    sqlx::query(&sql).execute(pool).await?;

    Ok(())
}

pub async fn create_schema(pool: &PgPool, table_name: &str) -> Result<(), sqlx::Error> {
    let sql_create_table = format!("
CREATE TABLE IF NOT EXISTS {0}
(
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY,
    date character(10) NOT NULL,
    name character varying(40) NOT NULL,
    is_off boolean NOT NULL,
    CONSTRAINT {0}_pkey PRIMARY KEY (id)
)
", table_name);
    let sql_create_index = format!("
CREATE UNIQUE INDEX IF NOT EXISTS {0}_unique_date ON {0} (date)
", table_name);

    let mut tx = pool.begin().await?;
    sqlx::query(&sql_create_table)
        .execute(&mut tx)
        .await?;
    sqlx::query(&sql_create_index)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;

    Ok(())
}

pub async fn insert_row(pool: &PgPool, table_name: &str, day: Day) -> Result<(), sqlx::Error> {
    let sql = format!("
INSERT INTO {} (date, name, is_off)
VALUES ($1, $2, $3)
ON CONFLICT(date) DO UPDATE
SET date=$1, name=$2, is_off=$3
", table_name);
    sqlx::query(&sql)
        .bind(day.date)
        .bind(day.name)
        .bind(day.is_off)
        .execute(pool)
        .await?;

    Ok(())
}


#[derive(Default, Debug, Clone, PartialEq)]
pub struct Day {
    pub date: String,
    pub name: String,
    pub is_off: bool
}
