use crate::users::models::Link;
use lambda_http::tracing::error;
use regex::Regex;

/// Never panics: a pattern that fails to compile (e.g. corrupted/legacy data
/// that slipped past validation at write time) is treated as "matches
/// nothing" rather than bringing down the whole guild's verify flow.
pub fn link_arr_match(links: &[Link], pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(regex) => links
            .iter()
            .any(|l| l.active && regex.is_match(&l.link_address)),
        Err(e) => {
            error!("Invalid verify pattern {pattern:?}: {e}");
            false
        }
    }
}

pub fn link_match(link: &Link, pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(regex) => link.active && regex.is_match(&link.link_address),
        Err(e) => {
            error!("Invalid verify pattern {pattern:?}: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(address: &str, active: bool) -> Link {
        Link {
            link_address: address.to_string(),
            linked_at: 0,
            active,
        }
    }

    #[test]
    fn link_match_returns_false_for_invalid_pattern_instead_of_panicking() {
        let l = link("https://example.com/user", true);
        // `(` is not a valid regex — this must not panic downstream matching.
        assert!(!link_match(&l, "("));
    }

    #[test]
    fn link_arr_match_returns_false_for_invalid_pattern_instead_of_panicking() {
        let links = vec![link("https://example.com/user", true)];
        // `(` is not a valid regex — this must not panic downstream matching.
        assert!(!link_arr_match(&links, "("));
    }

    #[test]
    fn link_match_still_matches_valid_pattern() {
        let l = link("https://example.com/user", true);
        assert!(link_match(&l, r"^https://example\.com/.*$"));
    }
}
