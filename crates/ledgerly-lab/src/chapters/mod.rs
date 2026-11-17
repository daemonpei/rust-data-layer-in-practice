//! One module per chapter. See README.md for the chapter-to-file map.

///  1. The Path a Write Actually Takes
pub mod ch01;

///  2. Sequential I/O, Random I/O and fsync
pub mod ch02;

///  3. B+tree vs LSM: Two Philosophies of Indexing
pub mod ch03;

///  4. WAL, MVCC and Isolation Levels
pub mod ch04;

///  5. Buffer Pool, Execution Plans and Statistics
pub mod ch05;

///  6. The Rust Data Access Landscape in 2026
pub mod ch06;

///  7. Toasty in Depth: an Application-Level Query Engine
pub mod ch07;

///  8. One Schema, Four Listings
pub mod ch08;

///  9. The Verdict: SQLx in Production, and Four Cases Against It
pub mod ch09;

/// 10. Project Skeleton and the Feature Matrix
pub mod ch10;

/// 11. Four Query APIs and Where They Stop
pub mod ch11;

/// 12. Compile-Time Verification: How It Works, What It Costs, and CI
pub mod ch12;

/// 13. sqlx.toml: The New Config System in 0.9
pub mod ch13;

/// 14. Dynamic SQL: QueryBuilder and SqlSafeStr
pub mod ch14;

/// 15. Error Handling: The Full sqlx::Error Picture
pub mod ch15;

/// 16. Type / Encode / Decode / FromRow
pub mod ch16;

/// 17. Custom Types
pub mod ch17;

/// 18. Time, UUIDs, Decimal, JSONB, and NULL
pub mod ch18;

/// 19. PostgreSQL-Specific Capabilities
pub mod ch19;

/// 20. MySQL and SQLite: Differences and Traps
pub mod ch20;

/// 21. Connection Pools: Parameters and Capacity Math
pub mod ch21;

/// 22. Transactions, Savepoints, and the Async Traps
pub mod ch22;

/// 23. Concurrency Correctness: Locks, Upserts, and Idempotency
pub mod ch23;

/// 24. Large Result Sets: Streaming, Cursors, and Keyset Pagination
pub mod ch24;

/// 25. Timeouts, Cancellation, and Retries
pub mod ch25;

/// 26. Bulk Writes and High-Throughput Pipelines
pub mod ch26;

/// 27. Migration Files and the sqlx-cli Workflow
pub mod ch27;

/// 28. Zero-Downtime Migrations: expand–migrate–contract
pub mod ch28;

/// 29. Online DDL and Lock Conflicts
pub mod ch29;

/// 30. Orchestrating Many Environments, Tenants and Databases
pub mod ch30;

/// 31. Backfills at Scale
pub mod ch31;

/// 32. Deployment Shapes: Containers, Kubernetes, Serverless, and PgBouncer
pub mod ch32;

/// 33. Observability: tracing, Metrics, and Slow Queries
pub mod ch33;

/// 34. Troubleshooting Handbook, Part 1: Connections and Pools
pub mod ch34;

/// 35. Troubleshooting Handbook, Part 2: Transactions, Migrations, and Drivers
pub mod ch35;

/// 36. Performance Engineering
pub mod ch36;

/// 37. Security
pub mod ch37;

/// 38. Read/Write Splitting and Replica Routing
pub mod ch38;

/// 39. Sharding
pub mod ch39;

/// 40. The Truth About Distributed Transactions
pub mod ch40;

/// 41. Caching and Consistency
pub mod ch41;

/// 42. CDC, Archival and Storage Tiers
pub mod ch42;

/// 43. Test Strategy and the Transaction Rollback Fixture
pub mod ch43;

/// 44. Version Upgrades and Dependency Governance
pub mod ch44;

/// 45. The Production Launch Checklist
pub mod ch45;
