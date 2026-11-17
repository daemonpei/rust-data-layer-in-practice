//! The small harness mentioned in the Preface.
//!
//! `cargo run --example bench` compares one-INSERT-per-row with chunked
//! multi-row INSERTs against the in-memory SQLite fixture. It prints numbers
//! for *your* machine. Nothing in the book quotes these as a benchmark.

use std::time::Instant;

use ledgerly_lab::{chapters::ch26, sqlite_memory};

const ROWS: usize = 2_000;
const BATCH: usize = 100;

async fn row_by_row(ids: &[String]) -> anyhow::Result<std::time::Duration> {
    let pool = sqlite_memory().await?;
    let start = Instant::now();
    for (i, id) in ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO usage_events \
             (event_id, tenant_id, metric, amount, occurred_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind("t-1")
        .bind("api_calls")
        .bind(1_i64)
        .bind(i as i64)
        .execute(&pool)
        .await?;
    }
    Ok(start.elapsed())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ids: Vec<String> = (0..ROWS).map(|i| format!("e{i}")).collect();

    let one = row_by_row(&ids).await?;
    let pool = sqlite_memory().await?;
    let start = Instant::now();
    let written = ch26::bulk_insert(&pool, &ids, BATCH).await?;
    let many = start.elapsed();

    let verify: i64 = sqlx::query_scalar("SELECT count(*) FROM usage_events")
        .fetch_one(&pool)
        .await?;
    assert_eq!(written as i64, verify);
    assert_eq!(verify, ROWS as i64);

    let per_sec = |d: std::time::Duration| ROWS as f64 / d.as_secs_f64();
    println!("rows                : {written}");
    println!("one INSERT per row  : {one:>10.2?}  ({:>9.0} rows/s)", per_sec(one));
    println!("chunked, batch {BATCH:<3}   : {many:>10.2?}  ({:>9.0} rows/s)", per_sec(many));
    println!("speedup             : {:>10.2}x", one.as_secs_f64() / many.as_secs_f64());
    println!("(in-memory SQLite; run on your own hardware before quoting a number)");
    Ok(())
}
