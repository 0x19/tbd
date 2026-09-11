//! Translation of the store's filters into SQL parameters, kept apart from
//! the queries so it can be unit-tested against [`PathPattern::matches`].

use super::PathPattern;

/// Path filters as two arrays: exact paths for `= any($exact)` and escaped
/// prefixes for `like any($prefixes)`. `None` means no path filter at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathParams {
    /// Exact paths; `None` when there is no path filter.
    pub exact: Option<Vec<String>>,
    /// `LIKE` patterns with `\`, `%` and `_` escaped and a trailing `%`;
    /// `None` when there is no path filter.
    pub prefixes: Option<Vec<String>>,
}

impl PathParams {
    /// Split the patterns. With no patterns both arrays are `None` and the
    /// SQL predicate collapses to true.
    #[must_use]
    pub fn of(patterns: &[PathPattern]) -> Self {
        if patterns.is_empty() {
            return Self {
                exact: None,
                prefixes: None,
            };
        }
        let mut exact = Vec::new();
        let mut prefixes = Vec::new();
        for p in patterns {
            match p {
                PathPattern::Exact(path) => exact.push(path.clone()),
                PathPattern::Prefix(prefix) => prefixes.push(format!("{}%", escape_like(prefix))),
            }
        }
        Self {
            exact: Some(exact),
            prefixes: Some(prefixes),
        }
    }
}

/// Escape a literal for `LIKE` with the default `\` escape character.
#[must_use]
pub fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny `LIKE` evaluator for `prefix%` patterns with `\` escapes, so the
    /// translation is checked against [`PathPattern::matches`] without a database.
    fn like(pattern: &str, path: &str) -> bool {
        let mut literal = String::new();
        let mut chars = pattern.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    if let Some(n) = chars.next() {
                        literal.push(n);
                    }
                }
                '%' => {
                    assert!(chars.peek().is_none(), "only a trailing %");
                    return path.starts_with(&literal);
                }
                '_' => panic!("unescaped _ in {pattern}"),
                other => literal.push(other),
            }
        }
        path == literal
    }

    #[test]
    fn escapes_the_like_wildcards() {
        assert_eq!(escape_like("traits.a_b"), "traits.a\\_b");
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a\\b"), "a\\\\b");
    }

    #[test]
    fn no_patterns_means_no_filter() {
        let p = PathParams::of(&[]);
        assert_eq!(p.exact, None);
        assert_eq!(p.prefixes, None);
    }

    #[test]
    fn prefixes_become_escaped_like_patterns() {
        let p = PathParams::of(&[
            PathPattern::Prefix("traits.a_b.".into()),
            PathPattern::Exact("profile.name".into()),
        ]);
        assert_eq!(p.exact, Some(vec!["profile.name".into()]));
        assert_eq!(p.prefixes, Some(vec!["traits.a\\_b.%".into()]));
    }

    proptest::proptest! {
        #[test]
        fn like_translation_agrees_with_matches(
            prefix in "[a-z0-9_]{1,6}(\\.[a-z0-9_]{1,6}){0,2}",
            path in "[a-z0-9_]{1,6}(\\.[a-z0-9_]{1,6}){0,3}",
        ) {
            let pattern = PathPattern::parse(&format!("{prefix}.*")).unwrap_or(PathPattern::Exact(String::new()));
            let params = PathParams::of(std::slice::from_ref(&pattern));
            let like_pattern = params.prefixes.and_then(|p| p.into_iter().next()).unwrap_or_default();
            proptest::prop_assert_eq!(like(&like_pattern, &path), pattern.matches(&path));
        }
    }
}
