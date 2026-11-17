//! Chapter 27 — Migration ordering and checksums.
//! The real files in migrations/ are read at test time, so a deleted or
//! renamed migration fails the suite.

use std::path::{Path, PathBuf};

pub fn migrations_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations")
}

pub fn list_migrations() -> anyhow::Result<Vec<String>> {
    let mut names: Vec<String> = std::fs::read_dir(migrations_dir())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".sql"))
        .collect();
    names.sort();
    Ok(names)
}

/// The numeric prefix is the version. sqlx-cli uses the same convention.
pub fn version_of(filename: &str) -> anyhow::Result<u64> {
    let prefix: String = filename.chars().take_while(|c| c.is_ascii_digit()).collect();
    if prefix.is_empty() { anyhow::bail!("no numeric version in {filename}"); }
    Ok(prefix.parse::<u64>()?)
}

/// A checksum changes when an applied migration is edited behind sqlx's back.
pub fn checksum(sql: &str) -> u64 {
    let mut h = 1469598103934665603_u64;
    for b in sql.bytes() { h ^= b as u64; h = h.wrapping_mul(1099511628211); }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_three_migrations_are_present_and_ordered() {
        let names = list_migrations().unwrap();
        assert_eq!(names.len(), 3);
        assert!(names[0].contains("tenants"));
        assert!(names[1].contains("usage_events"));
        assert!(names[2].contains("outbox"));
        assert_eq!(version_of(&names[0]).unwrap(), 20260101000001);
    }

    #[test]
    fn versions_are_strictly_increasing() {
        let versions: Vec<u64> = list_migrations().unwrap().iter()
            .map(|n| version_of(n).unwrap()).collect();
        assert!(versions.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn editing_an_applied_file_changes_the_checksum() {
        assert_ne!(checksum("CREATE TABLE t (id int);"),
                   checksum("CREATE TABLE t (id bigint);"));
        assert!(version_of("readme.md").is_err());
    }
}
