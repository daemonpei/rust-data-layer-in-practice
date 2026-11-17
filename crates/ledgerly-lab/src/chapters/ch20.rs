//! Chapter 20 — Backend differences that bite during a port.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend { Postgres, MySql, Sqlite }

impl Backend {
    pub fn placeholder(self, index: usize) -> String {
        match self {
            Backend::Postgres => format!("${index}"),
            Backend::MySql | Backend::Sqlite => "?".to_owned(),
        }
    }
    pub fn supports_returning(self) -> bool {
        matches!(self, Backend::Postgres | Backend::Sqlite)
    }
    pub fn has_native_uuid(self) -> bool { self == Backend::Postgres }
    pub fn ddl_locks_on_add_column(self) -> bool { self == Backend::MySql }
}

/// Rewrite a statement written for Postgres into the target dialect.
pub fn port(sql: &str, to: Backend) -> String {
    let mut out = sql.to_owned();
    for i in (1..=9).rev() {
        out = out.replace(&format!("${i}"), &to.placeholder(i));
    }
    if !to.supports_returning() {
        if let Some(pos) = out.find(" RETURNING ") {
            out.truncate(pos);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_the_first_porting_trap() {
        assert_eq!(Backend::Postgres.placeholder(3), "$3");
        assert_eq!(Backend::Sqlite.placeholder(3), "?");
        assert_eq!(Backend::MySql.placeholder(3), "?");
    }

    #[test]
    fn port_rewrites_and_drops_unsupported_clauses() {
        let sql = "INSERT INTO t (a, b) VALUES ($1, $2) RETURNING id";
        assert_eq!(port(sql, Backend::Postgres), sql);
        assert_eq!(port(sql, Backend::Sqlite),
                   "INSERT INTO t (a, b) VALUES (?, ?) RETURNING id");
        assert_eq!(port(sql, Backend::MySql),
                   "INSERT INTO t (a, b) VALUES (?, ?)");
    }

    #[test]
    fn capability_flags_differ() {
        assert!(Backend::Postgres.has_native_uuid());
        assert!(!Backend::Sqlite.has_native_uuid());
        assert!(Backend::MySql.ddl_locks_on_add_column());
    }
}
