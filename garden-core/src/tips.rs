use chrono::Datelike;

/// Parse an RFC3339 timestamp and return its day ordinal (days since 1 CE).
/// Returns 0 on parse failure (callers pass chrono::Utc::now().to_rfc3339()).
pub fn days_from_rfc3339(s: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.date_naive().num_days_from_ce() as i64)
        .unwrap_or(0)
}

/// Deterministically pick a tip index in `0..len` for a given day.
pub fn daily_tip_index(day_ordinal: i64, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        day_ordinal.rem_euclid(len as i64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_day_ordinal() {
        let d = days_from_rfc3339("2026-01-02T00:00:00+00:00");
        assert!(d > 0);
        assert_eq!(days_from_rfc3339("not-a-date"), 0);
    }

    #[test]
    fn consecutive_days_differ_by_one() {
        let a = days_from_rfc3339("2026-01-02T00:00:00+00:00");
        let b = days_from_rfc3339("2026-01-03T00:00:00+00:00");
        assert_eq!(b - a, 1);
    }

    #[test]
    fn index_wraps_within_len() {
        assert_eq!(daily_tip_index(0, 3), 0);
        assert_eq!(daily_tip_index(1, 3), 1);
        assert_eq!(daily_tip_index(3, 3), 0);
        assert_eq!(daily_tip_index(4, 3), 1);
        assert_eq!(daily_tip_index(-1, 3), 2);
    }

    #[test]
    fn empty_list_returns_zero() {
        assert_eq!(daily_tip_index(10, 0), 0);
    }

    #[test]
    fn known_date_matches_epoch() {
        // 1970-01-01 is day 719163 in the proleptic Gregorian calendar (chrono).
        let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        assert_eq!(epoch.num_days_from_ce(), 719163);
    }
}