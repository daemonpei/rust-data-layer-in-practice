//! Chapter 29 — Which DDL takes which lock, and how PostgreSQL 16 behaves.
//! Older version numbers here are historical markers (see the Preface):
//! they say when a behaviour appeared, not what to install.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock { AccessExclusive, ShareUpdateExclusive, MetadataOnly }

#[derive(Debug, Clone, Copy)]
pub enum Operation { AddColumnConstDefault, AddColumnNullable, SetNotNull, CreateIndex }

/// `pg_major` is the server version. The baseline in this book is 16.
pub fn lock_for(op: Operation, pg_major: u32) -> Lock {
    match op {
        Operation::AddColumnNullable => Lock::MetadataOnly,
        Operation::CreateIndex => Lock::ShareUpdateExclusive,
        Operation::AddColumnConstDefault => {
            if pg_major >= 11 { Lock::MetadataOnly } else { Lock::AccessExclusive }
        }
        Operation::SetNotNull => {
            if pg_major >= 12 { Lock::MetadataOnly } else { Lock::AccessExclusive }
        }
    }
}

/// The conflict matrix from the chapter, as a function rather than a 7-column
/// table that would be unreadable on a 6-inch screen.
pub fn blocks_writes(lock: Lock) -> bool { lock == Lock::AccessExclusive }

pub fn create_index_sql(concurrent: bool) -> String {
    if concurrent {
        "CREATE INDEX CONCURRENTLY usage_events_metric_idx ON usage_events (metric)".into()
    } else {
        "CREATE INDEX usage_events_metric_idx ON usage_events (metric)".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_postgres_16_a_constant_default_no_longer_rewrites_the_table() {
        assert_eq!(lock_for(Operation::AddColumnConstDefault, 16), Lock::MetadataOnly);
        assert_eq!(lock_for(Operation::AddColumnConstDefault, 10), Lock::AccessExclusive);
    }

    #[test]
    fn pg_12_is_the_set_not_null_marker() {
        assert_eq!(lock_for(Operation::SetNotNull, 12), Lock::MetadataOnly);
        assert_eq!(lock_for(Operation::SetNotNull, 11), Lock::AccessExclusive);
    }

    #[test]
    fn concurrent_index_builds_do_not_block_writes() {
        assert!(!blocks_writes(lock_for(Operation::CreateIndex, 16)));
        assert!(create_index_sql(true).contains("CONCURRENTLY"));
    }
}
