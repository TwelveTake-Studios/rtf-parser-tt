use std::panic;

use rtf_parser_tt::RtfDocument;

fn text(rtf: &str) -> String {
    RtfDocument::try_from(rtf).expect("document should parse").get_text()
}

fn does_not_panic(rtf: &str) {
    let owned = rtf.to_string();
    let outcome = panic::catch_unwind(move || RtfDocument::try_from(owned.as_str()).map(|d| d.get_text()));
    assert!(outcome.is_ok(), "parser panicked on {rtf:?}");
}

// Passing today

#[test]
fn keeps_text_after_control_word_delimiter() {
    assert_eq!(text(r"{\rtf1\ansi \b bold\b0  plain}"), "bold plain");
}

#[test]
fn keeps_raw_utf8_text() {
    assert_eq!(text("{\\rtf1\\ansi Привет мир}"), "Привет мир");
}

#[test]
fn keeps_field_result_only() {
    assert_eq!(text(r#"{\rtf1\ansi{\field{\*\fldinst HYPERLINK "http://x"}{\fldrslt link}}}"#), "link");
}

#[test]
fn skips_unknown_ignorable_destination() {
    assert_eq!(text(r"{\rtf1\ansi A{\*\unknown junk}B}"), "AB");
}

// Breaks

#[test]
#[ignore = "phase 1: \\par"]
fn par_ends_a_paragraph() {
    assert_eq!(text(r"{\rtf1\ansi First paragraph.\par Second paragraph.\par}"), "First paragraph.\nSecond paragraph.\n");
}

#[test]
#[ignore = "phase 1: \\page"]
fn page_breaks() {
    assert_eq!(text(r"{\rtf1\ansi A\page B}"), "A\nB");
}

#[test]
#[ignore = "phase 1: tables"]
fn table_cells_are_separated() {
    assert_eq!(text(r"{\rtf1\ansi\trowd\cellx1000\cellx2000 A\cell B\cell\row}"), "A\tB\n");
}

// Text that must not be lost

#[test]
#[ignore = "phase 1: whitespace-only text"]
fn keeps_whitespace_only_group() {
    assert_eq!(text(r"{\rtf1\ansi {\b bold}{ }{\i ital}}"), "bold ital");
}

#[test]
#[ignore = "phase 1: control symbols"]
fn decodes_control_symbols() {
    assert_eq!(text(r"{\rtf1\ansi A\~B C\-D E\_F}"), "A\u{a0}B C\u{ad}D E\u{2011}F");
}

#[test]
#[ignore = "phase 1: control word delimiters"]
fn keeps_punctuation_after_control_word() {
    assert_eq!(text(r"{\rtf1\ansi \b Bold\b0, then}"), "Bold, then");
}

#[test]
#[ignore = "phase 1: \\uN fallback"]
fn unicode_with_question_mark_fallback() {
    assert_eq!(text(r"{\rtf1\ansi \uc1舒? end}"), "\u{2014} end");
}

#[test]
#[ignore = "phase 1: control word delimiters"]
fn control_word_followed_by_non_ascii() {
    assert_eq!(text("{\\rtf1\\ansi \\iРим\\i0 x}"), "Рим x");
}

// Code pages

#[test]
#[ignore = "phase 1: code pages"]
fn decodes_cp1252_hex_escapes() {
    assert_eq!(
        text(r"{\rtf1\ansi\ansicpg1252 \'93Hi\'94 \'96 \'97 \'85 \'80}"),
        "\u{201c}Hi\u{201d} \u{2013} \u{2014} \u{2026} \u{20ac}"
    );
}

#[test]
#[ignore = "phase 1: code pages"]
fn decodes_cp1251_hex_escapes() {
    assert_eq!(text(r"{\rtf1\ansi\ansicpg1251 \'cf\'f0\'e8\'e2\'e5\'f2}"), "Привет");
}

// Content that is not body text

#[test]
#[ignore = "phase 1: destinations"]
fn skips_picture_data() {
    assert_eq!(text(r"{\rtf1\ansi Before{\pict\pngblip 89504e470d0a}After}"), "BeforeAfter");
}

#[test]
#[ignore = "phase 1: destinations"]
fn skips_info_group() {
    assert_eq!(text(r"{\rtf1\ansi{\info{\title Secret Title}{\author Bob}}Body}"), "Body");
}

#[test]
#[ignore = "phase 1: hidden text"]
fn skips_hidden_text() {
    assert_eq!(text(r"{\rtf1\ansi Shown{\v Hidden}}"), "Shown");
}

// Paragraph properties

#[test]
#[ignore = "phase 1: indents"]
fn applies_indents() {
    let doc = RtfDocument::try_from(r"{\rtf1\ansi \pard\li720\ri360\fi-180 Indented\par}").unwrap();
    let indent = doc.body[0].paragraph.indent;
    assert_eq!((indent.left, indent.right, indent.first_line), (720, 360, -180));
}

// No panics

#[test]
#[ignore = "phase 1: no panics"]
fn unpaired_surrogate_does_not_panic() {
    does_not_panic(r"{\rtf1\ansi \u-10179 x}");
}

#[test]
#[ignore = "phase 1: no panics"]
fn truncated_hex_escape_does_not_panic() {
    does_not_panic(r"{\rtf1\ansi \'5}");
}

#[test]
#[ignore = "phase 1: no panics"]
fn control_word_against_non_ascii_does_not_panic() {
    does_not_panic("{\\rtf1\\ansi \\iРим}");
}

#[test]
#[ignore = "phase 1: no panics"]
fn invalid_escape_bytes_do_not_panic() {
    does_not_panic(&String::from_utf8_lossy(&[b'{', 92, 39, 0, 10, 0, b'}']));
}
