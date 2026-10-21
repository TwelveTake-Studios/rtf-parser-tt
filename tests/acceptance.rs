use std::panic;

use rtf_parser_tt::{Lexer, LexerError, Parser, ParserError, RtfDocument, RtfError, TextOptions};

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
    assert_eq!(text("{\\rtf1\\ansi \u{41F}\u{440}\u{438}\u{432}\u{435}\u{442} \u{43C}\u{438}\u{440}}"), "\u{41F}\u{440}\u{438}\u{432}\u{435}\u{442} \u{43C}\u{438}\u{440}");
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
fn par_ends_a_paragraph() {
    assert_eq!(text(r"{\rtf1\ansi First paragraph.\par Second paragraph.\par}"), "First paragraph.\nSecond paragraph.\n");
}

#[test]
fn page_breaks() {
    assert_eq!(text(r"{\rtf1\ansi A\page B}"), "A\nB");
}

#[test]
fn table_cells_are_separated() {
    assert_eq!(text(r"{\rtf1\ansi\trowd\cellx1000\cellx2000 A\cell B\cell\row}"), "A\tB\n");
}

// Text that must not be lost

#[test]
fn keeps_whitespace_only_group() {
    assert_eq!(text(r"{\rtf1\ansi {\b bold}{ }{\i ital}}"), "bold ital");
}

#[test]
fn decodes_control_symbols() {
    assert_eq!(text(r"{\rtf1\ansi A\~B C\-D E\_F}"), "A\u{a0}B C\u{ad}D E\u{2011}F");
}

#[test]
fn keeps_punctuation_after_control_word() {
    assert_eq!(text(r"{\rtf1\ansi \b Bold\b0, then}"), "Bold, then");
}

#[test]
fn unicode_with_question_mark_fallback() {
    assert_eq!(text(r"{\rtf1\ansi \uc1\u8212? end}"), "\u{2014} end");
}

#[test]
fn control_word_followed_by_non_ascii() {
    assert_eq!(text("{\\rtf1\\ansi \\i\u{420}\u{438}\u{43C}\\i0  x}"), "\u{420}\u{438}\u{43C} x");
}

// Code pages

#[test]
fn decodes_cp1252_hex_escapes() {
    assert_eq!(
        text(r"{\rtf1\ansi\ansicpg1252 \'93Hi\'94 \'96 \'97 \'85 \'80}"),
        "\u{201c}Hi\u{201d} \u{2013} \u{2014} \u{2026} \u{20ac}"
    );
}

#[test]
fn decodes_cp1251_hex_escapes() {
    assert_eq!(text(r"{\rtf1\ansi\ansicpg1251 \'cf\'f0\'e8\'e2\'e5\'f2}"), "\u{41F}\u{440}\u{438}\u{432}\u{435}\u{442}");
}

// Content that is not body text

#[test]
fn skips_picture_data() {
    assert_eq!(text(r"{\rtf1\ansi Before{\pict\pngblip 89504e470d0a}After}"), "BeforeAfter");
}

#[test]
fn skips_info_group() {
    assert_eq!(text(r"{\rtf1\ansi{\info{\title Secret Title}{\author Bob}}Body}"), "Body");
}

#[test]
fn skips_hidden_text() {
    assert_eq!(text(r"{\rtf1\ansi Shown{\v Hidden}}"), "Shown");
}

#[test]
fn hidden_text_on_request() {
    let doc = RtfDocument::try_from(r"{\rtf1\ansi Shown{\v Hidden} again}").unwrap();
    assert_eq!(doc.get_text_with(TextOptions::default().hidden_text(true)), "ShownHidden again");
}

// Paragraph properties

#[test]
fn applies_indents() {
    let doc = RtfDocument::try_from(r"{\rtf1\ansi \pard\li720\ri360\fi-180 Indented\par}").unwrap();
    let indent = doc.body[0].paragraph.indent;
    assert_eq!((indent.left, indent.right, indent.first_line), (720, 360, -180));
}

// No panics

#[test]
fn unpaired_surrogate_does_not_panic() {
    does_not_panic(r"{\rtf1\ansi \u-10179 x}");
}

#[test]
fn truncated_hex_escape_does_not_panic() {
    does_not_panic(r"{\rtf1\ansi \'5}");
}

#[test]
fn control_word_against_non_ascii_does_not_panic() {
    does_not_panic("{\\rtf1\\ansi \\i\u{420}\u{438}\u{43C}}");
}

#[test]
fn invalid_escape_bytes_do_not_panic() {
    does_not_panic(&String::from_utf8_lossy(&[b'{', 92, 39, 0, 10, 0, b'}']));
}

#[test]
fn errors_are_send_and_sync() {
    fn error<E: std::error::Error + Send + Sync + 'static>() {}
    error::<RtfError>();
    error::<LexerError>();
    error::<ParserError>();
    let lexer_error = Lexer::scan(r"{\rtf1 \'zz}").unwrap_err();
    assert!(matches!(RtfError::from(lexer_error), RtfError::Lexer(_)));
    let parser_error = Parser::new(Lexer::scan(r"{\rtf1\fs99999999 x}").unwrap()).parse().unwrap_err();
    assert!(matches!(RtfError::from(parser_error), RtfError::Parser(_)));
    let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(RtfDocument::try_from(r"{\rtf1 \'zz}").unwrap_err());
    assert!(!boxed.to_string().is_empty());
}
