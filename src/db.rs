use sqlx::postgres::PgPool;
use sqlx::AssertSqlSafe;
use std::ops::DerefMut;

// validate that a table name contains only letters, digits, and underscores, and does not start with a digit
// should be called at the entry-point functions before any SQL containing the table name is constructed
fn validate_table_name(name: &str) -> Result<(), sqlx::Error> {
    let mut bytes = name.bytes();

    match bytes.next() {
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {}
        _ => {
            return Err(sqlx::Error::InvalidArgument(format!(
                "invalid table name: {name}"
            )));
        }
    }

    if !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err(sqlx::Error::InvalidArgument(format!(
            "invalid table name: {name}"
        )));
    }

    Ok(())
}

pub async fn check_table_exist(pool: &PgPool, table_name: &str) -> Result<bool, sqlx::Error> {
    validate_table_name(table_name)?;

    let exists = sqlx::query_scalar::<sqlx::Postgres, bool>(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = current_schema()
              AND table_type = 'BASE TABLE'
              AND table_name = $1
        )",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

pub async fn drop_table(pool: &PgPool, table_name: &str) -> Result<(), sqlx::Error> {
    validate_table_name(table_name)?;

    let sql = format!("DROP TABLE IF EXISTS {table_name}");
    sqlx::query(AssertSqlSafe(sql)).execute(pool).await?;

    Ok(())
}

pub async fn create_table(pool: &PgPool, table_name: &str) -> Result<(), sqlx::Error> {
    validate_table_name(table_name)?;

    let sql_create_table = format!(
        "CREATE TABLE IF NOT EXISTS {table_name} (\
         id integer NOT NULL GENERATED ALWAYS AS IDENTITY, \
         date character(10) NOT NULL, \
         name character varying(40) NOT NULL, \
         is_off boolean NOT NULL, \
         CONSTRAINT {table_name}_pkey PRIMARY KEY (id))"
    );
    let sql_create_index = format!(
        "CREATE UNIQUE INDEX IF NOT EXISTS {table_name}_unique_date ON {table_name} (date)"
    );

    let mut tx = pool.begin().await?;
    sqlx::query(AssertSqlSafe(sql_create_table))
        .execute(tx.deref_mut())
        .await?;
    sqlx::query(AssertSqlSafe(sql_create_index))
        .execute(tx.deref_mut())
        .await?;
    tx.commit().await?;

    Ok(())
}

// insert or update a holiday row, caller guarantees `table_name` has been validated
pub async fn insert_row(
    pool: &PgPool,
    table_name: &str,
    date: &str,
    name: &str,
    is_off: bool,
) -> Result<(), sqlx::Error> {
    let sql = format!(
        "INSERT INTO {table_name} (date, name, is_off) \
         VALUES ($1, $2, $3) \
         ON CONFLICT(date) DO UPDATE SET name=$2, is_off=$3"
    );
    sqlx::query(AssertSqlSafe(sql))
        .bind(date)
        .bind(name)
        .bind(is_off)
        .execute(pool)
        .await?;

    Ok(())
}
