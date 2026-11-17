# Rust Data Layer in Practice — companion code

Runnable code for **Rust Data Layer in Practice**
by Horace Pei. One module per chapter, `ch01` through `ch45`, each with tests.

Repository: <https://github.com/daemonpei/rust-data-layer-in-practice> · Issues: <https://github.com/daemonpei/rust-data-layer-in-practice/issues>

Most chapters run against an in-memory SQLite database. `cargo test` needs no
server, no Docker and no network once the crates are fetched. PostgreSQL 16
examples live in `ch19` and in `migrations/`; their integration tests are
marked `#[ignore]` and are described below.

## Quick start

```bash
git clone https://github.com/daemonpei/rust-data-layer-in-practice
cd rust-data-layer-in-practice
cargo test                         # unit + SQLite tests, no server required
cargo test -- --ignored            # PostgreSQL tests, needs DATABASE_URL
```

The pinned baseline is **SQLx 0.9.0**, **PostgreSQL 16**, **Rust 1.94+**
(developed and tested on 1.99.1), edition 2024. See `Cargo.toml` and
`crates/ledgerly-lab/sqlx.toml`.

## PostgreSQL 16 integration tests

```bash
docker compose up -d db
cp .env.example .env
cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres  # optional
sqlx migrate run            # optional: applies migrations/
DATABASE_URL=postgres://ledgerly:ledgerly@localhost:5432/ledgerly \
    cargo test -- --ignored
```

`docker-compose.yml` starts `postgres:16` with `pg_stat_statements` loaded and
`max_connections=200`. The ignored tests create their own tables, so they work
on a fresh database. Without Docker they stay skipped, and the rest of the
suite still runs.

## Layout

```
Cargo.toml                  workspace, pinned dependencies
docker-compose.yml          PostgreSQL 16
.env.example                DATABASE_URL
migrations/                 three PostgreSQL migrations (tenants, usage_events, outbox)
crates/ledgerly-lab/
  sqlx.toml                 SQLx 0.9 config, exercised by ch13
  src/lib.rs                crate root
  src/db.rs                 SQLite in-memory fixture shared by the chapters
  src/chapters/ch01.rs ... ch45.rs
```

## Chapter map

`cargo test chNN` runs the tests for one chapter (for example
`cargo test ch14` for the QueryBuilder chapter).

| Ch | Chapter | Module | What the code demonstrates |
|---:|---|---|---|
| 1 | The Path a Write Actually Takes | [`chapters/ch01.rs`](crates/ledgerly-lab/src/chapters/ch01.rs) | Latency budget as a sum of five named layers |
| 2 | Sequential I/O, Random I/O and fsync | [`chapters/ch02.rs`](crates/ledgerly-lab/src/chapters/ch02.rs) | Sequential vs random I/O, group commit amortisation |
| 3 | B+tree vs LSM: Two Philosophies of Indexing | [`chapters/ch03.rs`](crates/ledgerly-lab/src/chapters/ch03.rs) | B+tree probe depth vs LSM read/write amplification |
| 4 | WAL, MVCC and Isolation Levels | [`chapters/ch04.rs`](crates/ledgerly-lab/src/chapters/ch04.rs) | Append-only WAL replay and MVCC snapshot visibility |
| 5 | Buffer Pool, Execution Plans and Statistics | [`chapters/ch05.rs`](crates/ledgerly-lab/src/chapters/ch05.rs) | Clock-sweep buffer eviction, index-vs-scan cost model |
| 6 | The Rust Data Access Landscape in 2026 | [`chapters/ch06.rs`](crates/ledgerly-lab/src/chapters/ch06.rs) | Four crates scored against explicit requirements |
| 7 | Toasty in Depth: an Application-Level Query Engine | [`chapters/ch07.rs`](crates/ledgerly-lab/src/chapters/ch07.rs) | Predicate pushdown and residual filters |
| 8 | One Schema, Four Listings | [`chapters/ch08.rs`](crates/ledgerly-lab/src/chapters/ch08.rs) | One read expressed four ways |
| 9 | The Verdict: SQLx in Production, and Four Cases Against It | [`chapters/ch09.rs`](crates/ledgerly-lab/src/chapters/ch09.rs) | A recommendation with four documented exceptions |
| 10 | Project Skeleton and the Feature Matrix | [`chapters/ch10.rs`](crates/ledgerly-lab/src/chapters/ch10.rs) | Feature matrix and the pinned version baseline |
| 11 | Four Query APIs and Where They Stop | [`chapters/ch11.rs`](crates/ledgerly-lab/src/chapters/ch11.rs) | query / query_as / query_scalar against SQLite |
| 12 | Compile-Time Verification: How It Works, What It Costs, and CI | [`chapters/ch12.rs`](crates/ledgerly-lab/src/chapters/ch12.rs) | SqlSafeStr and AssertSqlSafe, the 0.9 safety boundary |
| 13 | sqlx.toml: The New Config System in 0.9 | [`chapters/ch13.rs`](crates/ledgerly-lab/src/chapters/ch13.rs) | Parsing the real sqlx.toml from the crate root |
| 14 | Dynamic SQL: QueryBuilder and SqlSafeStr | [`chapters/ch14.rs`](crates/ledgerly-lab/src/chapters/ch14.rs) | QueryBuilder IN lists and optional filters |
| 15 | Error Handling: The Full sqlx::Error Picture | [`chapters/ch15.rs`](crates/ledgerly-lab/src/chapters/ch15.rs) | Classifying sqlx::Error into an action |
| 16 | Type / Encode / Decode / FromRow | [`chapters/ch16.rs`](crates/ledgerly-lab/src/chapters/ch16.rs) | FromRow, column defaults, missing columns |
| 17 | Custom Types | [`chapters/ch17.rs`](crates/ledgerly-lab/src/chapters/ch17.rs) | A validated newtype with #[sqlx(transparent)] |
| 18 | Time, UUIDs, Decimal, JSONB, and NULL | [`chapters/ch18.rs`](crates/ledgerly-lab/src/chapters/ch18.rs) | UUID, JSON, money as minor units, and NULL |
| 19 | PostgreSQL-Specific Capabilities | [`chapters/ch19.rs`](crates/ledgerly-lab/src/chapters/ch19.rs) | PostgreSQL upsert, JSONB and SKIP LOCKED |
| 20 | MySQL and SQLite: Differences and Traps | [`chapters/ch20.rs`](crates/ledgerly-lab/src/chapters/ch20.rs) | Placeholder and RETURNING differences across backends |
| 21 | Connection Pools: Parameters and Capacity Math | [`chapters/ch21.rs`](crates/ledgerly-lab/src/chapters/ch21.rs) | The five pool capacity formulas from Appendix C |
| 22 | Transactions, Savepoints, and the Async Traps | [`chapters/ch22.rs`](crates/ledgerly-lab/src/chapters/ch22.rs) | Commit, rollback and savepoints |
| 23 | Concurrency Correctness: Locks, Upserts, and Idempotency | [`chapters/ch23.rs`](crates/ledgerly-lab/src/chapters/ch23.rs) | Idempotency keys and atomic ON CONFLICT accumulation |
| 24 | Large Result Sets: Streaming, Cursors, and Keyset Pagination | [`chapters/ch24.rs`](crates/ledgerly-lab/src/chapters/ch24.rs) | Keyset pagination and OFFSET cost |
| 25 | Timeouts, Cancellation, and Retries | [`chapters/ch25.rs`](crates/ledgerly-lab/src/chapters/ch25.rs) | Retryable failures, jittered backoff, write safety |
| 26 | Bulk Writes and High-Throughput Pipelines | [`chapters/ch26.rs`](crates/ledgerly-lab/src/chapters/ch26.rs) | Bind-parameter cap and chunked bulk inserts |
| 27 | Migration Files and the sqlx-cli Workflow | [`chapters/ch27.rs`](crates/ledgerly-lab/src/chapters/ch27.rs) | Migration ordering, versions and checksums |
| 28 | Zero-Downtime Migrations: expand–migrate–contract | [`chapters/ch28.rs`](crates/ledgerly-lab/src/chapters/ch28.rs) | expand / backfill / contract state machine |
| 29 | Online DDL and Lock Conflicts | [`chapters/ch29.rs`](crates/ledgerly-lab/src/chapters/ch29.rs) | DDL lock modes and the PG 11, 12, 16 markers |
| 30 | Orchestrating Many Environments, Tenants and Databases | [`chapters/ch30.rs`](crates/ledgerly-lab/src/chapters/ch30.rs) | Tenant slug validation and routing strategies |
| 31 | Backfills at Scale | [`chapters/ch31.rs`](crates/ledgerly-lab/src/chapters/ch31.rs) | Resumable watermark backfill with lag throttling |
| 32 | Deployment Shapes: Containers, Kubernetes, Serverless, and PgBouncer | [`chapters/ch32.rs`](crates/ledgerly-lab/src/chapters/ch32.rs) | PgBouncer modes vs session features |
| 33 | Observability: tracing, Metrics, and Slow Queries | [`chapters/ch33.rs`](crates/ledgerly-lab/src/chapters/ch33.rs) | Span naming, slow-query threshold, sampling |
| 34 | Troubleshooting Handbook, Part 1: Connections and Pools | [`chapters/ch34.rs`](crates/ledgerly-lab/src/chapters/ch34.rs) | Symptom-to-SQL decision tree for pool incidents |
| 35 | Troubleshooting Handbook, Part 2: Transactions, Migrations, and Drivers | [`chapters/ch35.rs`](crates/ledgerly-lab/src/chapters/ch35.rs) | SQLSTATE to action, migration recovery |
| 36 | Performance Engineering | [`chapters/ch36.rs`](crates/ledgerly-lab/src/chapters/ch36.rs) | Plan cost estimates and stale statistics |
| 37 | Security | [`chapters/ch37.rs`](crates/ledgerly-lab/src/chapters/ch37.rs) | Identifier whitelists, tenant predicates, masking |
| 38 | Read/Write Splitting and Replica Routing | [`chapters/ch38.rs`](crates/ledgerly-lab/src/chapters/ch38.rs) | Read/write routing and the consistency window |
| 39 | Sharding | [`chapters/ch39.rs`](crates/ledgerly-lab/src/chapters/ch39.rs) | Tenant sharding and remapping cost |
| 40 | The Truth About Distributed Transactions | [`chapters/ch40.rs`](crates/ledgerly-lab/src/chapters/ch40.rs) | Transactional outbox state machine and reconciliation |
| 41 | Caching and Consistency | [`chapters/ch41.rs`](crates/ledgerly-lab/src/chapters/ch41.rs) | Cache-aside with TTL jitter and invalidation order |
| 42 | CDC, Archival and Storage Tiers | [`chapters/ch42.rs`](crates/ledgerly-lab/src/chapters/ch42.rs) | LSN watermark and storage tiers |
| 43 | Test Strategy and the Transaction Rollback Fixture | [`chapters/ch43.rs`](crates/ledgerly-lab/src/chapters/ch43.rs) | Transaction-rollback test fixture |
| 44 | Version Upgrades and Dependency Governance | [`chapters/ch44.rs`](crates/ledgerly-lab/src/chapters/ch44.rs) | semver, MSRV and staged upgrades |
| 45 | The Production Launch Checklist | [`chapters/ch45.rs`](crates/ledgerly-lab/src/chapters/ch45.rs) | The nine-item launch checklist as code |

## How this relates to the book

The listings in the book are trimmed to fit a page. These are the runnable
forms, rewritten from an internal project so they are self-contained. They are
not production code copied verbatim, and they are deliberately smaller than a
real service. Where a chapter describes PostgreSQL behaviour that SQLite
cannot reproduce (JSONB operators, lock modes, `FOR UPDATE SKIP LOCKED`), the
module keeps the real SQL as a constant and exercises the surrounding logic;
the ignored tests run the SQL against PostgreSQL 16.

## Errata

Corrections and version drift are recorded in [ERRATA.md](ERRATA.md).
Please report a broken listing to 1091186233@qq.com with the chapter number
and the compiler output.

## License

MIT. See [LICENSE](LICENSE).
