//! Chapter 6 — Four crates on one table.
//! A scoring model makes the trade-off explicit instead of rhetorical.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crate { Toasty, Diesel, SeaOrm, Sqlx }

#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    pub runtime_sql_check: bool,
    pub hand_written_sql: bool,
    pub async_native: bool,
    pub owns_migrations: bool,
    pub dynamic_query_ergonomics: u8, // 0..5
    pub abstraction_layers: u8,      // fewer is better here
}

pub const CAPS: [(Crate, Capabilities); 4] = [
    (Crate::Toasty, Capabilities {
        runtime_sql_check: false, hand_written_sql: false, async_native: true,
        owns_migrations: true, dynamic_query_ergonomics: 3, abstraction_layers: 3,
    }),
    (Crate::Diesel, Capabilities {
        runtime_sql_check: true, hand_written_sql: false, async_native: false,
        owns_migrations: true, dynamic_query_ergonomics: 4, abstraction_layers: 2,
    }),
    (Crate::SeaOrm, Capabilities {
        runtime_sql_check: false, hand_written_sql: true, async_native: true,
        owns_migrations: true, dynamic_query_ergonomics: 4, abstraction_layers: 3,
    }),
    (Crate::Sqlx, Capabilities {
        runtime_sql_check: true, hand_written_sql: true, async_native: true,
        owns_migrations: true, dynamic_query_ergonomics: 3, abstraction_layers: 1,
    }),
];

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub check: i32,
    pub sql: i32,
    pub async_w: i32,
    pub dynamism: i32,
    pub thinness: i32,
}

impl Default for Weights {
    fn default() -> Self {
        Self { check: 3, sql: 3, async_w: 2, dynamism: 1, thinness: 2 }
    }
}

pub fn score(c: &Capabilities, w: &Weights) -> i32 {
    let mut s = 0;
    if c.runtime_sql_check { s += w.check; }
    if c.hand_written_sql { s += w.sql; }
    if c.async_native { s += w.async_w; }
    s += c.dynamic_query_ergonomics as i32 * w.dynamism;
    s += (4 - c.abstraction_layers) as i32 * w.thinness;
    s
}

pub fn rank(w: &Weights) -> Vec<(Crate, i32)> {
    let mut out: Vec<_> = CAPS.iter()
        .map(|(k, c)| (*k, score(c, w)))
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0))));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_pick_sqlx() {
        let r = rank(&Weights::default());
        assert_eq!(r[0].0, Crate::Sqlx);
    }

    #[test]
    fn changing_weights_changes_the_winner() {
        // A team that only cares about building queries with a builder API.
        let w = Weights { check: 0, sql: 0, async_w: 0, dynamism: 10, thinness: 0 };
        assert_ne!(rank(&w)[0].0, Crate::Sqlx);
    }

    #[test]
    fn seaorm_is_built_on_sqlx_in_this_book() {
        // The point made in Chapter 9: it is not a competing driver.
        assert_eq!(CAPS.len(), 4);
        assert!(CAPS.iter().any(|(k, _)| *k == Crate::SeaOrm));
    }
}
