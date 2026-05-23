# AGENTS.md -- holiday-cli

## Purpose

Fetch Chinese public holiday data from a remote JSON API and sync it into a local PostgreSQL table.

## Architecture

Three modules under `src/`:

| Module | Responsibility |
|--------|---------------|
| `main.rs` | CLI entry point: reads env config, parses CLI args, orchestrates the sync loop |
| `db.rs` | All PostgreSQL operations: schema check, table create/drop, row insert |
| `data.rs` | HTTP fetch from the holiday JSON API and serde models |

The data flow is `main` to `data` to `main` to `db` -- `data` and `db` never talk to each other.

## Tech Stack

- **Runtime**: `tokio` (multi-threaded, `#[tokio::main]`)
- **Database**: PostgreSQL via `sqlx 0.9` with `runtime-tokio` + `tls-rustls-ring-webpki`
- **HTTP**: `reqwest 0.13` with `rustls` TLS (no `blocking`, no `native-tls`)
- **Serialization**: `serde` + `serde_json` (transitive via `reqwest/json`)
- **Date/time**: `chrono`
- **Config**: `dotenv` from `.env`
- **MSRV**: Rust 1.94

## Environment Variables

All defined in `.env`, loaded by `dotenv`:

| Variable | Example | Purpose |
|----------|---------|---------|
| `DATABASE_URL` | `postgres://user:pass@localhost:5432/db` | PostgreSQL connection string |
| `DATA_URL_PREFIX` | `https://raw.githubusercontent.com/NateScarlet/holiday-cn/master/` | Base URL for JSON data files |
| `TABLE_NAME` | `holiday` | Target table name (validated at runtime) |

## CLI Usage

```
cargo run -- [begin_year] [end_year]
```

- No arguments: syncs 2007 through current year.
- One argument: syncs `begin_year` through current year.
- Two arguments: syncs `begin_year` through `end_year` (inclusive).
- `cargo run -- 0`: drops the table and exits. Re-run normally to recreate and reload.

## Build

```bash
cargo build
```

No compile-time database connection is required. The project uses `sqlx::query_scalar` and `sqlx::query` exclusively (no `query!` macro, no `SQLX_OFFLINE` data).

Lint and format before committing:

```bash
cargo fmt
cargo clippy
```

## Key Design Decisions

### No `query!` macro

`query!` requires a live PostgreSQL connection at compile time (or `SQLX_OFFLINE` metadata). For a small CLI tool this is unnecessary build friction. All queries use `sqlx::query_scalar` or `sqlx::query`, which are checked only at runtime.

### Table name safety: validate then `AssertSqlSafe`

PostgreSQL does not allow identifiers as bind parameters, so the table name from `TABLE_NAME` must be interpolated into SQL strings. `sqlx 0.9` requires dynamic SQL strings to be wrapped in `AssertSqlSafe`.

The project uses a layered approach:

1. `validate_table_name()` checks that the string matches `[A-Za-z_][A-Za-z0-9_]*` and returns `Err` otherwise.
2. Entry-point functions (`check_table_exist`, `create_table`, `drop_table`) call `validate_table_name` internally.
3. `insert_row` does **not** re-validate -- it is called in a tight loop and the doc comment declares the trust assumption.

The validator lives in `src/db.rs` and is `pub(crate)` so no caller can bypass it.

### No intermediate `Day` struct in `db.rs`

`data::Day` is the canonical model (from the JSON API). `insert_row` takes individual field values (`date: &str`, `name: &str`, `is_off: bool`) rather than a struct. This avoids duplicating the model and eliminates mapping code in `main.rs`.

### Minimal dependency features

Both `sqlx` and `reqwest` use `default-features = false` to avoid pulling in unused drivers (`mysql`, `sqlite`), TLS backends (`native-tls`), and tooling (`macros`, `migrate`). The `tokio` dependency uses `features = ["macros", "rt-multi-thread"]` instead of `"full"`.

## Naming

- `drop_table` and `create_table` -- not `drop_schema` / `create_schema`. In PostgreSQL a *schema* is a namespace, not a table. These functions operate on tables.

## Code Style

- `cargo fmt` is authoritative. Run it before committing.
- `cargo clippy` must pass with zero warnings.
- No emoji in code comments, doc comments, or documentation files.
- Use Markdown inline code markers (`` ` ``) for all technical identifiers: type names, function names, file paths, command-line arguments, and environment variable names. This applies to `AGENTS.md`, `README.md`, `INIT.md`, `PLAN.md`, and any future design documents.
- Prefer prose over tables. Use tables only when they are the clearest way to present structured information.
- All documentation must be concise, clear, and to the point.
