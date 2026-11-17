//! Chapter 4 — WAL and MVCC.
//! A minimal append-only log plus the snapshot visibility rule.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalRecord {
    pub lsn: u64,
    pub xid: u64,
    pub page_id: u64,
    pub payload: String,
}

#[derive(Debug, Default)]
pub struct Wal {
    records: Vec<WalRecord>,
}

impl Wal {
    pub fn append(&mut self, xid: u64, page_id: u64, payload: &str) -> u64 {
        let lsn = self.records.len() as u64 + 1;
        self.records.push(WalRecord {
            lsn, xid, page_id, payload: payload.to_owned(),
        });
        lsn
    }

    /// Replay stops at the last committed xid. Uncommitted tails are discarded.
    pub fn replay(&self, committed: &[u64]) -> Vec<&WalRecord> {
        self.records.iter()
            .filter(|r| committed.contains(&r.xid))
            .collect()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}

/// True when the tuple version is visible to a snapshot taken at
/// `snapshot_xid`. `active` lists transactions still running at that moment.
///
/// xmin = creator, xmax = deleter (None means the row is live).
pub fn visible(xmin: u64, xmax: Option<u64>, snapshot_xid: u64, active: &[u64]) -> bool {
    if xmin >= snapshot_xid || active.contains(&xmin) { return false; }
    match xmax {
        None => true,
        Some(deleter) => deleter >= snapshot_xid || active.contains(&deleter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_drops_the_uncommitted_tail() {
        let mut wal = Wal::default();
        wal.append(41, 7, "insert");
        wal.append(42, 7, "uncommitted update");
        assert_eq!(wal.len(), 2);
        let applied = wal.replay(&[41]);
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].lsn, 1);
    }

    #[test]
    fn a_reader_never_sees_a_concurrent_writer() {
        assert!(visible(40, None, 50, &[41]));
        assert!(!visible(41, None, 50, &[41]));
        assert!(!visible(51, None, 50, &[41]));
    }

    #[test]
    fn a_commit_after_the_snapshot_does_not_hide_the_row() {
        assert!(visible(40, Some(55), 50, &[]));
        assert!(!visible(40, Some(45), 50, &[]));
    }
}
