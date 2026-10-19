use rtf_parser_tt::RtfDocument;

fn parse(rtf: &str) -> Result<String, String> {
    RtfDocument::try_from(rtf).map(|d| d.get_text()).map_err(|e| e.to_string())
}

#[test]
fn unpaired_surrogate_becomes_replacement_character() {
    assert_eq!(parse(r"{\rtf1 a\u-10179 b}").unwrap(), "a\u{FFFD}b");
}

#[test]
fn surrogate_pair_still_decodes() {
    assert_eq!(parse(r"{\rtf1 \u-10179\u-8704 }").unwrap(), "\u{1F600}");
}

#[test]
fn surrogate_pair_split_by_unicode_skip_count_decodes() {
    assert_eq!(parse(r"{\rtf1\uc0 \u-10180\uc1 \u-8264\'3f x}").unwrap(), "\u{1F3B8} x");
    assert_eq!(parse(r"{\rtf1 Emoji: \uc0 \u-10180\uc1 \u-8264\'3f\uc0 \u-10179\uc1 \u-8704\'3f}").unwrap(), "Emoji: \u{1F3B8}\u{1F600}");
}

#[test]
fn high_surrogate_followed_by_text_becomes_replacement_character() {
    assert_eq!(parse(r"{\rtf1\uc0 \u-10180 x}").unwrap(), "\u{FFFD}x");
    assert_eq!(parse(r"{\rtf1\uc0 \u-10180}").unwrap(), "\u{FFFD}");
}

#[test]
fn truncated_hex_escape_is_an_error() {
    assert!(parse(r"{\rtf1 a\'5}").is_err());
    assert!(parse(r"{\'9}").is_err());
}

#[test]
fn hex_escape_followed_by_multibyte_text_is_an_error() {
    assert!(parse("{\\rtf1 \\'9\u{420}}").is_err());
}

#[test]
fn control_word_followed_by_non_ascii_does_not_panic() {
    assert!(parse("{\\rtf1 \\i\u{420}\u{438}\u{43C}}").is_ok());
    assert!(parse("{\\rtf1 \\b\u{85} text}").is_ok());
    assert!(parse("{\\rtf1 \\b\u{a0}text}").is_ok());
    assert!(parse("{\\rtf1\u{85}}").is_ok());
}

#[test]
fn ascii_whitespace_still_ends_a_control_word() {
    assert_eq!(parse("{\\rtf1 {\\b bold}\\tab\tx}").unwrap(), "bold\tx");
    assert_eq!(parse("{\\rtf1 {\\b bold}\\tab\u{b}x}").unwrap(), "bold\tx");
}

#[test]
fn extreme_unicode_skip_counts_do_not_panic() {
    assert_eq!(parse(r"{\rtf1\uc-5 \u233 x}").unwrap(), "éx");
    assert_eq!(parse(r"{\rtf1\uc2000000000 \u233 x}").unwrap(), "éx");
}
