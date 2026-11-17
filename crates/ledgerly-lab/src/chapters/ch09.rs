//! Chapter 9 — The decision and its four exceptions.
//! A recommendation that cannot say when it is wrong is not a recommendation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Case {
    SqlIsAnAsset,
    CompileTimeChecks,
    ThinLayer,
    ReportHeavyCrud,
    NeedPortableDsl,
    TeamPrefersEntities,
    SyncOnlyRuntime,
}

#[derive(Debug, Clone, Copy)]
pub struct Requirements {
    pub dbas_hand_you_sql: bool,
    pub schema_is_stable_at_compile_time: bool,
    pub mostly_crud_screens: bool,
    pub must_swap_backends: bool,
    pub team_wants_entities: bool,
    pub async_runtime: bool,
}

pub fn recommend(r: &Requirements) -> &'static str {
    if !r.async_runtime { return "Diesel (sync) or diesel-async with an explicit runtime"; }
    if r.must_swap_backends { return "Diesel: its DSL is backend-portable"; }
    if r.mostly_crud_screens && r.team_wants_entities { return "SeaORM"; }
    if !r.dbas_hand_you_sql && !r.schema_is_stable_at_compile_time {
        return "Toasty or a document store: compile-time SQL checks buy little here";
    }
    "SQLx"
}

/// The four cases from the closing section of the chapter.
pub fn cases_against_sqlx() -> [&'static str; 4] {
    [
        "Hundreds of near-identical CRUD screens: an entity layer saves real work.",
        "The schema does not exist at compile time (tenant-defined columns).",
        "The same code must run unchanged on three SQL dialects.",
        "A synchronous codebase with no async runtime and no plan to add one.",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typical() -> Requirements {
        Requirements {
            dbas_hand_you_sql: true, schema_is_stable_at_compile_time: true,
            mostly_crud_screens: false, must_swap_backends: false,
            team_wants_entities: false, async_runtime: true,
        }
    }

    #[test]
    fn the_ledgerly_requirements_pick_sqlx() {
        assert_eq!(recommend(&typical()), "SQLx");
    }

    #[test]
    fn each_exception_changes_the_answer() {
        let mut r = typical(); r.async_runtime = false;
        assert_ne!(recommend(&r), "SQLx");
        let mut r = typical(); r.must_swap_backends = true;
        assert_ne!(recommend(&r), "SQLx");
        let mut r = typical(); r.mostly_crud_screens = true; r.team_wants_entities = true;
        assert_ne!(recommend(&r), "SQLx");
        let mut r = typical(); r.dbas_hand_you_sql = false;
        r.schema_is_stable_at_compile_time = false;
        assert_ne!(recommend(&r), "SQLx");
    }

    #[test]
    fn the_book_lists_four_exceptions_not_zero() {
        assert_eq!(cases_against_sqlx().len(), 4);
        let _ = Case::SqlIsAnAsset;
    }
}
