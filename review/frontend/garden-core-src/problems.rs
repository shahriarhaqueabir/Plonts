use crate::models::ProblemCause;

/// Reorder causes so any cause whose name mentions a known plant pest or disease
/// ranks first (stable), leaving the curated order otherwise untouched.
pub fn rank_causes(
    causes: Vec<ProblemCause>,
    plant_pests: &[String],
    plant_diseases: &[String],
) -> Vec<ProblemCause> {
    let matches = |c: &ProblemCause| -> bool {
        let name = c.name.to_lowercase();
        plant_pests
            .iter()
            .chain(plant_diseases)
            .filter(|t| !t.trim().is_empty())
            .any(|t| name.contains(&t.to_lowercase()))
    };
    let mut ranked = Vec::with_capacity(causes.len());
    let mut rest = Vec::new();
    for c in causes {
        if matches(&c) {
            ranked.push(c);
        } else {
            rest.push(c);
        }
    }
    ranked.extend(rest);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProblemCause;

    fn cause(id: &str, name: &str) -> ProblemCause {
        ProblemCause {
            id: id.into(),
            name: name.into(),
            description: "desc".into(),
            treatment: "treat".into(),
            prevention: "prev".into(),
            organic: true,
            image_asset_id: None,
        }
    }

    #[test]
    fn matching_cause_moves_first() {
        let causes = vec![
            cause("a", "Nitrogen deficiency"),
            cause("b", "Aphids"),
            cause("c", "Powdery mildew"),
        ];
        let ranked = rank_causes(causes, &["aphids".to_string()], &[]);
        assert_eq!(ranked[0].id, "b");
        assert_eq!(ranked[1].id, "a");
        assert_eq!(ranked[2].id, "c");
    }

    #[test]
    fn no_matches_preserves_order() {
        let causes = vec![cause("a", "Nitrogen deficiency"), cause("b", "Aphids")];
        let ranked = rank_causes(causes, &["slugs".to_string()], &[]);
        assert_eq!(ranked[0].id, "a");
        assert_eq!(ranked[1].id, "b");
    }
}
