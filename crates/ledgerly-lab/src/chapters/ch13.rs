//! Chapter 13 — sqlx.toml, the config file introduced by SQLx 0.9.
//! The real file lives at the crate root and is embedded here at compile time
//! so these tests fail if somebody deletes it.

pub const CONFIG: &str = include_str!("../../sqlx.toml");

/// Minimal reader for the keys used by this project. SQLx itself parses the
/// whole file with `sqlx-core::config`; this is deliberately small so the
/// chapter stays readable.
pub fn value(section: &str, key: &str) -> Option<String> {
    let mut current = "";
    for raw in CONFIG.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() { continue; }
        if line.starts_with('[') && line.ends_with(']') {
            current = &line[1..line.len() - 1];
            continue;
        }
        if current != section { continue; }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().trim_matches('"').to_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_config_file_is_present_and_nonempty() {
        assert!(CONFIG.contains("[common]"));
        assert!(CONFIG.contains("[drivers.postgres]"));
    }

    #[test]
    fn macros_prefer_chrono_for_date_time_types() {
        assert_eq!(value("macros.preferred-crates", "date-time").as_deref(), Some("chrono"));
    }

    #[test]
    fn the_database_url_variable_is_the_default_one() {
        assert_eq!(value("common", "database-url-var").as_deref(), Some("DATABASE_URL"));
        assert_eq!(value("common", "not-a-key"), None);
    }
}
