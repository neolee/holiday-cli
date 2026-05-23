# sqlx 0.9 Upgrade Plan

## Goal

Upgrade `holiday-cli` from `sqlx 0.8.x` to `sqlx 0.9.x` while keeping the tool small, readable, and easy to verify.

The upgrade should focus on the actual compatibility issues introduced by `sqlx 0.9`:

- Runtime and TLS features are now separate.
- Dynamic SQL strings must be explicitly audited with `AssertSqlSafe`.
- `query!()` keeps a compile-time database dependency, which is unnecessary for this small CLI.

Non-goal: turn this project into a larger framework with migrations, custom error stacks, or broad refactors.

## Current Findings

### `generic-array`

Do not try to upgrade `generic-array` directly.

It is not a direct dependency of this project. It is pulled in through the `sqlx` crypto/hash dependency chain:

```
sqlx → sqlx-postgres → hmac / sha2 / md-5 → digest → block-buffer → generic-array
```

It is used internally for PostgreSQL SCRAM-SHA-256 authentication. The v0.14.7 → v0.14.9 bump is a patch-level change within the same minor semver range; `cargo update --verbose` lists it as available but unresolved because the upstream constraints (likely `^0.14`) already accept the newer version — Cargo simply hasn't been asked to resolve it yet. Upgrading `sqlx` may pull a newer `generic-array` version during resolution, but either way the patch-level changes are irrelevant to this project.

Recommended action: leave it alone unless a security audit reports a concrete issue. It requires no manual intervention.

### `sqlx`

`sqlx 0.9.x` is a real migration, not just a lockfile update. The current code uses dynamic SQL for table names, and `sqlx 0.9` deliberately rejects unaudited dynamic SQL strings.

The project should handle that by validating `TABLE_NAME` before it is interpolated into SQL, then using `AssertSqlSafe` only at the small number of audited dynamic SQL call sites.

## Change Scope

Expected files:

- `Cargo.toml`
- `src/db.rs`
- `src/main.rs`
- `Cargo.lock`

Keep the implementation direct. Avoid new modules unless the file grows substantially, which is unlikely.

## Cargo Changes

Use the split runtime/TLS features required by `sqlx 0.9`.

Recommended dependency shape:

```toml
[package]
name = "holiday"
version = "0.1.0"
edition = "2021"
rust-version = "1.94"

[dependencies]
chrono = "0.4"
dotenv = "0.15"
reqwest = { version = "0.13", default-features = false, features = ["json", "rustls"] }
serde = { version = "1.0", features = ["derive"] }
sqlx = { version = "0.9", default-features = false, features = [
    "runtime-tokio",
    "tls-rustls-ring-webpki",
    "postgres",
] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Notes:

- `runtime-tokio-rustls` must not be used with `sqlx 0.9`; it has been removed.
- `tls-rustls-ring-webpki` is explicit about the rustls provider and root store behavior.
- `default-features = false` keeps `sqlx` from enabling unused feature sets such as macros, migrations, or extra database support.
- `reqwest` no longer needs `blocking`; this code uses async requests.
- `tokio/full` is more than this CLI needs.
- Remove `serde_json` if no code directly references it after the change.
- Remove the old commented MySQL `sqlx` dependency lines.

## Database Code Changes

### 1. Validate table names before interpolation

Because PostgreSQL identifiers cannot be passed as bind parameters, table names must still be interpolated. Keep that safe by allowing only a simple ASCII identifier:

```text
[A-Za-z_][A-Za-z0-9_]*
```

Prefer a tiny hand-written validator over adding `regex`.

Recommended shape in `src/db.rs`:

```rust
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
```

Keep this validation inside `db.rs`, not only in `main.rs`. The `db` functions are public, and keeping validation near the SQL interpolation prevents future callers from bypassing the safety check.

Use a layered approach to avoid redundant checks:

- **Entry-point functions** (`check_table_exist`, `create_table`, `drop_table`): call `validate_table_name` internally as a safety gate.
- **`insert_row`**: do **not** validate — it is called in a tight loop (once per holiday row), and re-validating the same `table_name` hundreds of times is wasteful. Document the trust assumption with a doc comment: `/// Caller guarantees table_name has been validated.`

This gives defense in depth at the public API boundary without paying the validation cost on every row insert.

### 2. Use `AssertSqlSafe` only after validation

For dynamic SQL that includes the validated table name, use owned strings:

```rust
sqlx::query(sqlx::AssertSqlSafe(sql))
```

Avoid this form:

```rust
sqlx::query(sqlx::AssertSqlSafe(&sql))
```

`AssertSqlSafe<String>` is the clearer and safer fit here. Passing `&String` can run into trait/lifetime mismatches and is unnecessary.

The affected functions are:

- `drop_schema` (rename to `drop_table`)
- `create_schema` (rename to `create_table`)
- `insert_row`

Rename both for consistency: `drop_schema` → `drop_table` and `create_schema` → `create_table`. The current names are misleading in PostgreSQL terminology (a *schema* is a namespace that contains tables, not a table itself). Having one function named `drop_table` and another still named `create_schema` would be confusing. Rename both in the same commit and update all call sites in `main.rs`.

### 3. Replace `query!()` with `query_scalar`

`check_table_exist` currently uses `query!()`, which requires compile-time database access unless SQLx offline metadata is maintained. That is unnecessary for this CLI.

Use runtime query checking instead:

```rust
let exists = sqlx::query_scalar::<sqlx::Postgres, bool>(
    "
SELECT EXISTS (
    SELECT 1
    FROM information_schema.tables
    WHERE table_schema = current_schema()
      AND table_type = 'BASE TABLE'
      AND table_name = $1
)
",
)
.bind(table_name)
.fetch_one(pool)
.await?;
```

This also avoids the previous `Option<bool>` and `unwrap()`.

The `table_schema = current_schema()` predicate avoids incorrectly finding a same-named table in another schema.

### 4. Remove `db::Day`

`db::Day` duplicates the API model from `data::Day` with only a field rename. Keep `db.rs` as database operations and let `insert_row` receive the fields it inserts:

```rust
/// Insert or update a holiday row.
/// Caller guarantees `table_name` has been validated.
pub async fn insert_row(
    pool: &PgPool,
    table_name: &str,
    date: &str,
    name: &str,
    is_off: bool,
) -> Result<(), sqlx::Error>
```

Then `main.rs` can call:

```rust
db::insert_row(pool, table_name, &day.date, &day.name, day.is_off_day).await?;
```

This keeps the data flow simple and removes the intermediate struct.

## Main Code Changes

Keep `main.rs` changes narrow:

- Update calls for renamed database functions.
- Remove construction of `db::Day`.
- Do not add a second table-name validator in `main.rs`; `db.rs` owns validation near dynamic SQL.

If earlier failure is preferred, `main.rs` may call a public `db::validate_table_name(&table_name)?` after reading config. That is optional. If added, keep the same validator as the one used by `db.rs`; do not duplicate logic.

Be careful not to write:

```rust
let table_name = validate_table_name(&env::var("TABLE_NAME").expect("wrong table name"));
```

That borrows from a temporary `String`. Read the environment variable into a `String` first.

## Suggested Implementation Order

1. Update `Cargo.toml` dependencies and remove stale commented dependency lines.
2. Run `cargo update -p sqlx`.
3. Replace `query!()` in `check_table_exist` with `query_scalar`.
4. Add table-name validation in `db.rs`.
5. Wrap the validated dynamic SQL strings with `AssertSqlSafe(sql)`.
6. Rename `drop_schema` to `drop_table` and `create_schema` to `create_table`; update all call sites.
7. Remove `db::Day` and update `insert_row` plus its call site.
8. Run formatting and checks.

## Verification

Minimum local checks:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features
```

Functional smoke test with a real PostgreSQL database:

1. Set `DATABASE_URL`, `DATA_URL_PREFIX`, and `TABLE_NAME`.
2. Run the CLI for a small year range.
3. Run the same range again to verify `ON CONFLICT(date)` updates cleanly.
4. Run with argument `0` to verify table drop behavior.
5. Try an invalid `TABLE_NAME`, such as `foo;drop_table`, and confirm it fails before executing SQL.

## Rollback

The change should be easy to revert because it is limited to dependency declarations, `db.rs`, `main.rs`, and `Cargo.lock`.

If the upgrade causes runtime incompatibility with the target database or TLS environment, rollback should be:

```bash
git checkout -- Cargo.toml Cargo.lock src/db.rs src/main.rs
cargo check
```
