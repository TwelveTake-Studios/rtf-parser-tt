use rtf_parser_tt::RtfDocument;

fn bytes_text(rtf: &[u8]) -> String {
    RtfDocument::from_bytes(rtf).expect("document should parse").get_text()
}

fn emdash_then(fallback: &str) -> String {
    [r"{\rtf1\ansi\uc1", r"\u", "8212", fallback, "X}"].concat()
}

// Unicode fallback

#[test]
fn a_control_word_or_symbol_is_one_fallback_character() {
    for fallback in [r"\emdash ", r"\~", r"\-", r"\@", r"\*", "\\\n", r"\bin2 xy", r"\b ", r"\'97", r"\chftn "] {
        let rtf = emdash_then(fallback);
        let document = RtfDocument::try_from(rtf.as_str()).unwrap();
        assert_eq!(document.get_text(), "\u{2014}X", "fallback {fallback:?}");
        assert!(document.body.iter().all(|block| !block.painter.bold), "fallback {fallback:?}");
    }
}

#[test]
fn the_letter_after_a_one_character_fallback_is_kept() {
    let rtf = [r"{\rtf1\ansi\uc3", r"\u", r"8212\'2d\~\b X{\uc2", r"\u", "233?}Y}"].concat();
    assert_eq!(bytes_text(rtf.as_bytes()), "\u{2014}X\u{e9}Y");
}

#[test]
fn a_group_ends_the_fallback() {
    let rtf = [r"{\rtf1\ansi\uc2", r"\u", "8212{X}Y}"].concat();
    assert_eq!(bytes_text(rtf.as_bytes()), "\u{2014}XY");
}

#[test]
fn a_unicode_escape_is_never_taken_as_fallback() {
    let pair = [r"{\rtf1\ansi\uc1 a", r"\u", "55357", r"\u", "56447?b}"].concat();
    assert_eq!(bytes_text(pair.as_bytes()), "a\u{1f47f}b");
    let two = [r"{\rtf1\ansi\uc1 a", r"\u", "8212", r"\u", "8211?b}"].concat();
    assert_eq!(bytes_text(two.as_bytes()), "a\u{2014}\u{2013}b");
}

// Default font and font code page

#[test]
fn plain_restores_the_default_font() {
    let rtf = br"{\rtf1\ansi\deff1{\fonttbl{\f0\fcharset0 Latin;}{\f1\fcharset204 Cyrillic;}}\f0\'e9{\plain\'e9}\'e9\plain\'e9}";
    assert_eq!(bytes_text(rtf), "\u{e9}\u{439}\u{e9}\u{439}");
}

#[test]
fn font_code_page_overrides_the_character_set() {
    let rtf = br"{\rtf1\ansi\ansicpg1252{\fonttbl{\f0\fcharset0 Latin;}{\f1\fcharset0\cpg1251 Cyrillic;}}\f0\'e9{\f1\'e9}\'e9}";
    assert_eq!(bytes_text(rtf), "\u{e9}\u{439}\u{e9}");
}

#[test]
fn a_zero_font_code_page_is_ignored() {
    let rtf = br"{\rtf1\ansi{\fonttbl{\f0\fcharset204\cpg0 Cyrillic;}}\f0\'e9}";
    assert_eq!(bytes_text(rtf), "\u{439}");
}

#[test]
fn font_names_decode_through_the_font_code_page() {
    let rtf = br"{\rtf1\ansi{\fonttbl{\f0\cpg1251 \'c0\'f0\'e8\'e0\'eb;}}\f0 x}";
    let document = RtfDocument::from_bytes(rtf).unwrap();
    assert_eq!(document.header.font_table[&0].name, "\u{410}\u{440}\u{438}\u{430}\u{43b}");
    assert_eq!(document.header.font_table[&0].code_page, Some(1251));
}

// Code pages

#[test]
fn an_unknown_code_page_reads_as_windows_1252() {
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg777 \x80\\'80\x93}"), "\u{20ac}\u{20ac}\u{201c}");
}

#[cfg(feature = "encoding_rs")]
#[test]
fn koi8_and_iso_8859_code_pages() {
    let koi8 = b"{\\rtf1\\ansi\\ansicpg20866{\\fonttbl{\\f0\\fcharset0 Default;}{\\f1\\fcharset204 Cyrillic;}}\\f0 \xc1\\'c1{\\f1\\'c1}\\'c1}";
    assert_eq!(bytes_text(koi8), "\u{430}\u{430}\u{411}\u{430}");
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg21866 \xa4}"), "\u{454}");
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg28592 \xa1\\'a1}"), "\u{104}\u{104}");
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg28595 \xb0}"), "\u{410}");
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg28605 \xa4}"), "\u{20ac}");
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg10007 \x80}"), "\u{410}");
}

// Raw bytes and binary data

#[test]
fn a_literal_backslash_before_bin_is_text() {
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg1252 A\\\\bin9 \xe9Z}"), "A\\bin9 \u{e9}Z");
}

#[test]
fn a_literal_backslash_then_a_real_bin() {
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg1252 A\\\\\\bin3 }\xff{Z\xe9}"), "A\\Z\u{e9}");
}

#[test]
fn a_real_bin_still_drops_its_payload() {
    assert_eq!(bytes_text(b"{\\rtf1\\ansi A{\\pict\\bin2 \xff\xfe}B}"), "AB");
}

#[cfg(feature = "encoding_rs")]
#[test]
fn a_shift_jis_trail_byte_before_bin_is_text() {
    assert_eq!(bytes_text(b"{\\rtf1\\ansi\\ansicpg932 \x95\x5cbin9 Z\\bin3 abc!}"), "\u{8868}bin9 Z!");
}
