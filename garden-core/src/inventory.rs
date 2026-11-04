use crate::models::Viability;

/// Whole years a seed packet has been around, clamped at 0 (never negative).
pub fn years_old(year: i64, current_year: i64) -> i64 {
    (current_year - year).max(0)
}

/// Broad viability band for a seed of a given harvest/packing year.
pub fn viability_status(year: i64, current_year: i64) -> Viability {
    match years_old(year, current_year) {
        0..=2 => Viability::Fresh,
        3..=4 => Viability::Aging,
        _ => Viability::Expired,
    }
}

/// True when a tool's next maintenance date has arrived or passed
/// (ISO YYYY-MM-DD strings compare lexicographically == chronologically).
pub fn is_due(next_maintenance: Option<&str>, today: &str) -> bool {
    match next_maintenance {
        Some(d) if !d.trim().is_empty() => d.trim() <= today,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn years_old_clamps_negative() {
        assert_eq!(years_old(2026, 2024), 0);
        assert_eq!(years_old(2024, 2026), 2);
    }

    #[test]
    fn viability_bands() {
        assert_eq!(viability_status(2026, 2026), Viability::Fresh);
        assert_eq!(viability_status(2024, 2026), Viability::Fresh);
        assert_eq!(viability_status(2023, 2026), Viability::Aging);
        assert_eq!(viability_status(2022, 2026), Viability::Aging);
        assert_eq!(viability_status(2021, 2026), Viability::Expired);
    }

    #[test]
    fn due_compares_dates() {
        assert!(is_due(Some("2026-01-01"), "2026-06-01"));
        assert!(!is_due(Some("2026-12-31"), "2026-06-01"));
        assert!(is_due(Some("2026-06-01"), "2026-06-01"));
        assert!(!is_due(None, "2026-06-01"));
        assert!(!is_due(Some(""), "2026-06-01"));
    }
}
