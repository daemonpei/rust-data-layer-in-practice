//! Chapter 44 — A version bump is a checklist, not a Cargo.toml edit.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version { pub major: u32, pub minor: u32, pub patch: u32 }

pub fn parse(v: &str) -> anyhow::Result<Version> {
    let parts: Vec<&str> = v.trim_start_matches('v').split('.').collect();
    if parts.len() != 3 { anyhow::bail!("expected major.minor.patch, got {v}"); }
    Ok(Version { major: parts[0].parse()?, minor: parts[1].parse()?, patch: parts[2].parse()? })
}

pub fn is_breaking(from: &Version, to: &Version) -> bool {
    from.major != to.major || (from.major == 0 && from.minor != to.minor)
}

pub fn upgrade_steps(breaking: bool) -> &'static [&'static str] {
    if breaking {
        &["read the CHANGELOG", "check MSRV", "run the full test suite",
          "test migrations on a copy of production", "roll out to one instance"]
    } else {
        &["check MSRV", "run the full test suite"]
    }
}

pub fn msrv_ok(toolchain: &Version, required: &Version) -> bool {
    (toolchain.major, toolchain.minor) >= (required.major, required.minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_and_minor_bumps_are_not_breaking_after_1_0() {
        assert!(!is_breaking(&parse("0.9.0").unwrap(), &parse("0.9.1").unwrap()));
        assert!(is_breaking(&parse("0.8.6").unwrap(), &parse("0.9.0").unwrap()));
        assert!(!is_breaking(&parse("1.2.3").unwrap(), &parse("1.3.0").unwrap()));
        assert!(is_breaking(&parse("1.2.3").unwrap(), &parse("2.0.0").unwrap()));
    }

    #[test]
    fn a_breaking_bump_requires_a_staged_rollout() {
        assert_eq!(upgrade_steps(true).len(), 5);
        assert_eq!(upgrade_steps(false).len(), 2);
    }

    #[test]
    fn sqlx_0_9_needs_rust_1_94() {
        assert!(msrv_ok(&parse("1.99.1").unwrap(), &parse("1.94.0").unwrap()));
        assert!(!msrv_ok(&parse("1.85.0").unwrap(), &parse("1.94.0").unwrap()));
    }
}
