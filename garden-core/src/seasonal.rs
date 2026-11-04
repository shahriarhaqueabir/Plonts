use crate::models::{Hemisphere, MonthWindow, Season, Seasonality};

pub const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const MONTHS_NORTH: [Season; 12] = [
    Season::Winter, Season::Winter, Season::Spring, Season::Spring, Season::Spring,
    Season::Summer, Season::Summer, Season::Summer,
    Season::Autumn, Season::Autumn, Season::Autumn, Season::Winter,
];

pub fn month_from_name(s: &str) -> Option<i32> {
    let s = s.trim().to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }
    for (i, name) in MONTH_NAMES.iter().enumerate() {
        let full = name.to_ascii_lowercase();
        if full == s || (s.len() >= 3 && full.starts_with(&s[..3])) {
            return Some((i + 1) as i32);
        }
    }
    None
}

pub fn month_in_window(month0: i32, window: MonthWindow) -> bool {
    let start = (window.start_month - 1).rem_euclid(12);
    let end = (window.end_month - 1).rem_euclid(12);
    if start <= end {
        month0 >= start && month0 <= end
    } else {
        month0 >= start || month0 <= end
    }
}

pub fn season_name(s: Season) -> &'static str {
    match s {
        Season::Spring => "Spring",
        Season::Summer => "Summer",
        Season::Autumn => "Autumn",
        Season::Winter => "Winter",
    }
}

pub fn sowing_eligible(
    seasonality: Option<&Seasonality>,
    sowing_seasons: &[Season],
    hemisphere: Hemisphere,
    month0: i32,
) -> (bool, String) {
    if let Some(seas) = seasonality {
        if let Some(w) = seas.sowing.iter().find(|w| month_in_window(month0, **w)) {
            return (
                true,
                format!("Within the specific sowing window ({} to {}).", w.start_month, w.end_month),
            );
        }
    }

    let adjusted = if hemisphere == Hemisphere::South {
        (month0 + 6) % 12
    } else {
        month0
    };
    let season = MONTHS_NORTH[adjusted as usize];
    if sowing_seasons.contains(&season) {
        return (true, format!("Currently in the {} sowing window.", season_name(season)));
    }

    let specific = seasonality.and_then(|seas| {
        if seas.sowing.is_empty() {
            None
        } else {
            let ranges = seas
                .sowing
                .iter()
                .map(|w| format!("{} to {}", w.start_month, w.end_month))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("Too early or late. High-confidence window(s): {}.", ranges))
        }
    });

    let reason = specific.unwrap_or_else(|| {
        let allowed = sowing_seasons
            .iter()
            .map(|s| season_name(*s))
            .collect::<Vec<_>>()
            .join(" or ");
        format!("Too early or late. This plant prefers {}.", allowed)
    });

    (false, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_and_short_month_names() {
        assert_eq!(month_from_name("April"), Some(4));
        assert_eq!(month_from_name("apr"), Some(4));
        assert_eq!(month_from_name("September"), Some(9));
        assert_eq!(month_from_name("NotAMonth"), None);
    }

    #[test]
    fn window_without_wrap() {
        // March (2) through June (5)
        assert!(month_in_window(2, MonthWindow { start_month: 3, end_month: 6 }));
        assert!(!month_in_window(0, MonthWindow { start_month: 3, end_month: 6 }));
    }

    #[test]
    fn window_with_wrap() {
        // October (9) through March (2)
        assert!(month_in_window(10, MonthWindow { start_month: 10, end_month: 3 }));
        assert!(month_in_window(0, MonthWindow { start_month: 10, end_month: 3 }));
        assert!(!month_in_window(6, MonthWindow { start_month: 10, end_month: 3 }));
    }

    #[test]
    fn granular_window_wins() {
        let seas = Seasonality {
            sowing: vec![MonthWindow { start_month: 3, end_month: 4 }],
            harvest: vec![],
        };
        let (ok, _) = sowing_eligible(Some(&seas), &[], Hemisphere::North, 2);
        assert!(ok);
        let (ok2, _) = sowing_eligible(Some(&seas), &[], Hemisphere::North, 8);
        assert!(!ok2);
    }

    #[test]
    fn hemisphere_shift_flips_season() {
        // Northern Winter = January (0). Southern Winter = July (6).
        let (north, _) = sowing_eligible(None, &[Season::Summer], Hemisphere::North, 5);
        assert!(north);
        let (south, _) = sowing_eligible(None, &[Season::Summer], Hemisphere::South, 0);
        assert!(south); // January in the south is summer
    }
}
