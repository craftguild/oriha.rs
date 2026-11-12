#[path = "generated/shortcodes.rs"]
mod generated;

pub(crate) fn resolve_shortcode(shortcode: &str) -> Option<&'static str> {
    generated::EMOJI_SHORTCODES
        .binary_search_by(|(candidate, _)| candidate.cmp(&shortcode))
        .ok()
        .map(|index| generated::EMOJI_SHORTCODES[index].1)
}

#[cfg(test)]
mod tests {
    use super::resolve_shortcode;

    #[test]
    fn resolves_github_and_cldr_shortcodes() {
        assert_eq!(resolve_shortcode("+1"), Some("👍"));
        assert_eq!(resolve_shortcode("thumbsup"), Some("👍"));
        assert_eq!(resolve_shortcode("thumbs_up"), Some("👍"));
        assert_eq!(resolve_shortcode("thumbs_up_light_skin_tone"), Some("👍🏻"));
        assert_eq!(resolve_shortcode("rocket"), Some("🚀"));
    }

    #[test]
    fn leaves_unknown_shortcodes_unresolved() {
        assert_eq!(resolve_shortcode("not_a_real_shortcode"), None);
    }

    #[test]
    fn generated_table_is_sorted_and_unique() {
        for window in super::generated::EMOJI_SHORTCODES.windows(2) {
            assert!(
                window[0].0 < window[1].0,
                "emoji shortcode table must be sorted and unique: {:?} then {:?}",
                window[0],
                window[1]
            );
        }
    }
}
