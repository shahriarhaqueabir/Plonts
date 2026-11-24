use crate::models::{BedShape, LayoutPlant};

/// Grid snap interval in centimetres (10 cm).
pub const GRID_CM: f64 = 10.0;

/// Approximate mature spacing (circle diameter, cm) for a plant, keyed off its
/// primary category. Per-plant spacing data is a future content-pack addition;
/// this is the MVP fallback shared by the canvas renderer and warning engine.
/// Effective spacing (cm) for a plant: per-plant override when available, else the
/// category-based MVP fallback.
pub fn spacing_cm(category: &str, override_cm: Option<f64>) -> f64 {
    override_cm.unwrap_or_else(|| default_spacing_cm(category))
}

pub fn default_spacing_cm(category: &str) -> f64 {
    let c = category.to_ascii_lowercase();
    if c.contains("tree") {
        300.0
    } else if c.contains("shrub") {
        120.0
    } else if c.contains("herb") {
        25.0
    } else if c.contains("flower") {
        30.0
    } else if c.contains("indoor") {
        25.0
    } else {
        40.0
    }
}

/// Snap a centimetre coordinate to the nearest grid line.
pub fn snap_to_grid(cm: f64) -> f64 {
    (cm / GRID_CM).round() * GRID_CM
}

/// True when two spacing circles (radius = spacing / 2) intersect.
pub fn circles_overlap(x1: f64, y1: f64, r1: f64, x2: f64, y2: f64, r2: f64) -> bool {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dist = (dx * dx + dy * dy).sqrt();
    dist < r1 + r2
}

/// A plant is shade tolerant when its sunlight requirement mentions shade,
/// indirect light, or explicitly tolerates partial shade.
pub fn shade_tolerant(sunlight: Option<&str>) -> bool {
    let Some(s) = sunlight else { return false };
    let s = s.to_ascii_lowercase();
    s.contains("shade") || s.contains("indirect") || s.contains("tolerates_partial")
}

/// True when a sun-loving plant is placed in a shady bed.
pub fn shade_conflict(plant_sunlight: Option<&str>, bed_exposure: Option<&str>) -> bool {
    let Some(bed) = bed_exposure else { return false };
    bed.to_ascii_lowercase().contains("shade") && !shade_tolerant(plant_sunlight)
}

/// Index of the top-most bed containing the point (cm), or None.
pub fn hit_bed(beds: &[BedShape], x_cm: f64, y_cm: f64) -> Option<usize> {
    beds.iter()
        .enumerate()
        .rev()
        .find(|(_, b)| {
            x_cm >= b.x && x_cm <= b.x + b.width_cm && y_cm >= b.y && y_cm <= b.y + b.height_cm
        })
        .map(|(i, _)| i)
}

/// Index of the top-most plant whose centre is within `radius_cm` of the point.
pub fn hit_plant(plants: &[LayoutPlant], x_cm: f64, y_cm: f64, radius_cm: f64) -> Option<usize> {
    let r2 = radius_cm * radius_cm;
    plants
        .iter()
        .enumerate()
        .rev()
        .find(|(_, p)| {
            let dx = p.x - x_cm;
            let dy = p.y - y_cm;
            dx * dx + dy * dy <= r2
        })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BedShape, LayoutPlant};

    #[test]
    fn spacing_by_category() {
        assert_eq!(default_spacing_cm("fruit_tree"), 300.0);
        assert_eq!(default_spacing_cm("herb"), 25.0);
        assert_eq!(default_spacing_cm("flower"), 30.0);
        assert_eq!(default_spacing_cm("ornamental_shrub"), 120.0);
        assert_eq!(default_spacing_cm("vegetable"), 40.0);
    }

    #[test]
    fn snap_rounds_to_nearest_grid() {
        assert_eq!(snap_to_grid(13.0), 10.0);
        assert_eq!(snap_to_grid(16.0), 20.0);
        assert_eq!(snap_to_grid(0.0), 0.0);
        assert_eq!(snap_to_grid(10.0), 10.0);
    }

    #[test]
    fn overlapping_circles() {
        assert!(circles_overlap(0.0, 0.0, 20.0, 10.0, 10.0, 20.0));
    }

    #[test]
    fn non_overlapping_circles() {
        assert!(!circles_overlap(0.0, 0.0, 20.0, 50.0, 0.0, 20.0));
    }

    #[test]
    fn shade_tolerance_detection() {
        assert!(shade_tolerant(Some("partial_shade")));
        assert!(shade_tolerant(Some("bright_indirect_light")));
        assert!(!shade_tolerant(Some("full_sun")));
        assert!(!shade_tolerant(None));
    }

    #[test]
    fn shade_conflict_requires_shady_bed_and_sun_lover() {
        assert!(shade_conflict(Some("full_sun"), Some("shade")));
        assert!(shade_conflict(Some("full_sun"), Some("partial_shade")));
        assert!(!shade_conflict(Some("partial_shade"), Some("shade")));
        assert!(!shade_conflict(Some("full_sun"), None));
        assert!(!shade_conflict(Some("full_sun"), Some("full_sun")));
    }

    #[test]
    fn hit_bed_returns_topmost() {
        let beds = vec![
            BedShape { id: "a".into(), name: "A".into(), x: 0.0, y: 0.0, width_cm: 100.0, height_cm: 100.0, sun_exposure: None },
            BedShape { id: "b".into(), name: "B".into(), x: 50.0, y: 50.0, width_cm: 100.0, height_cm: 100.0, sun_exposure: None },
        ];
        assert_eq!(hit_bed(&beds, 75.0, 75.0), Some(1));
        assert_eq!(hit_bed(&beds, 10.0, 10.0), Some(0));
        assert_eq!(hit_bed(&beds, 500.0, 500.0), None);
    }

    #[test]
    fn hit_plant_within_radius() {
        let plants = vec![
            LayoutPlant {
                id: "p1".into(), layout_id: "l".into(), catalog_id: "c1".into(),
                plant_name: "A".into(), bed_id: None, x: 0.0, y: 0.0, quantity: 1, created_at: String::new(),
            },
        ];
        assert_eq!(hit_plant(&plants, 5.0, 5.0, 12.0), Some(0));
        assert_eq!(hit_plant(&plants, 50.0, 50.0, 12.0), None);
    }
}
