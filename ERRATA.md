# Errata

Version baseline: **SQLx 0.9.0 / PostgreSQL 16 / Rust 1.94+ (tested on 1.99.1) /
edition 2024**. First Edition, 2026.

No known corrections at the time of publication.

## Reporting a correction

Open an issue at <https://github.com/daemonpei/rust-data-layer-in-practice/issues> or mail `1091186233@qq.com` with:

1. the chapter number (or `migrations/`, `docker-compose.yml`),
2. the exact compiler error or wrong output,
3. your Rust version and operating system.

Corrections are added to the table below and, when they affect the book's
text, to the next print update. The book's version numbers are pinned on
purpose; a newer crate release is not automatically an error.

| Date | Chapter | What changed | Applies to |
|---|---|---|---|
| — | — | — | — |

## Known environmental differences

- `cargo test` uses in-memory SQLite. PostgreSQL-specific behaviour (JSONB
  operators, lock modes, `FOR UPDATE SKIP LOCKED`) is only exercised by the
  `#[ignore]`d tests with a real PostgreSQL 16 server.
- The performance constants in the chapter modules are comparison values,
  not benchmark results. See the Preface in the book.
