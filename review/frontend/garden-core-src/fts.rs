/// Build a safe FTS5 MATCH query from raw user input.
/// Splits into whitespace tokens, strips non-alphanumeric chars (removing FTS
/// operators like `"`, `*`, `-`, `(`, `)`, `:`, `^`), double-quotes each token,
/// and joins with AND. Returns "" when there are no usable tokens.
pub fn build_query(input: &str) -> String {
    input
        .split_whitespace()
        .map(|t| t.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(" AND ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_token() {
        assert_eq!(build_query("tomato"), "\"tomato\"");
    }

    #[test]
    fn multiple_tokens_are_anded() {
        assert_eq!(build_query("sweet pepper"), "\"sweet\" AND \"pepper\"");
    }

    #[test]
    fn strips_fts_operators() {
        assert_eq!(build_query("tom* \"x\" -y"), "\"tom\" AND \"x\" AND \"y\"");
    }

    #[test]
    fn empty_input() {
        assert_eq!(build_query("   "), "");
    }
}
