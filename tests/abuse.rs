use std::fs;
use std::panic;
use std::path::Path;
use std::time::{Duration, Instant};

use rtf_parser_tt::RtfDocument;

const BS: char = '\\';
const LIMIT: Duration = Duration::from_secs(30);

fn outcome(rtf: &str) -> Result<String, String> {
    let owned = rtf.to_string();
    match panic::catch_unwind(move || RtfDocument::try_from(owned.as_str()).map(|d| d.get_text()).map_err(|e| e.to_string())) {
        Ok(result) => result,
        Err(_) => panic!("parser panicked on {:?}", &rtf[..rtf.len().min(200)]),
    }
}

fn text(rtf: &str) -> String {
    outcome(rtf).unwrap_or_else(|e| panic!("expected text, got error {e} for {:?}", &rtf[..rtf.len().min(200)]))
}

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    assert!(elapsed < LIMIT, "{label} took {elapsed:?}");
    value
}

#[test]
fn deep_nesting() {
    let depth = 100_000;
    let rtf = format!("{{{BS}rtf1 {}x{}}}", "{".repeat(depth), "}".repeat(depth));
    assert_eq!(timed("deep nesting", || text(&rtf)), "x");
}

#[test]
fn unclosed_brace_bomb_is_an_error() {
    let rtf = format!("{{{BS}rtf1 {}", "{".repeat(200_000));
    assert!(timed("brace bomb", || outcome(&rtf)).is_err());
}

#[test]
fn unbalanced_braces_are_errors_not_panics() {
    assert!(outcome(&format!("{{{BS}rtf1 a}}}}}}}}")).is_err());
    assert!(outcome(&format!("}}{{{BS}rtf1 a}}")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1 a")).is_err());
    assert!(outcome("").is_err());
    assert!(outcome("}").is_err());
    assert!(outcome("{").is_err());
    assert_eq!(text("{}"), "");
}

#[test]
fn bin_payloads() {
    assert!(outcome(&format!("{{{BS}rtf1 a{BS}bin999999999 b}}")).is_err());
    assert_eq!(text(&format!("{{{BS}rtf1 a{BS}bin-5 b}}")), "ab");
    assert_eq!(text(&format!("{{{BS}rtf1 a{{{BS}pict{BS}bin3 }}{{}}}}b}}")), "ab");
    assert_eq!(text(&format!("{{{BS}rtf1 a{BS}bin1 \u{E9} b}}")), "a b");
    assert_eq!(text(&format!("{{{BS}rtf1 a{BS}bin0 b}}")), "ab");
}

#[test]
fn bin_payloads_in_raw_bytes() {
    let mut rtf = b"{\\rtf1 a{\\pict\\bin6 ".to_vec();
    rtf.extend_from_slice(&[0xFF, b'}', 0x00, b'{', 0x80, b'\\']);
    rtf.extend_from_slice(b"}b}");
    assert_eq!(RtfDocument::from_bytes(&rtf).unwrap().get_text(), "ab");
}

#[test]
fn unicode_skip_counts_at_the_limits() {
    assert_eq!(text(&format!("{{{BS}rtf1{BS}uc2147483647 {BS}u233 abc}}")), "\u{E9}");
    assert_eq!(text(&format!("{{{BS}rtf1{BS}uc-2147483648 {BS}u233 abc}}")), "\u{E9}abc");
    assert_eq!(text(&format!("{{{BS}rtf1{BS}uc3 {BS}u233 a}}b}}")), "\u{E9}b");
}

#[test]
fn unicode_values_at_the_limits() {
    let cases = [
        ("2147483647", "\u{FFFD}"),
        ("-2147483648", "\u{FFFD}"),
        ("-32768", "\u{8000}"),
        ("-1", "\u{FFFF}"),
        ("0", "\u{0}"),
        ("1114111", "\u{10FFFF}"),
        ("1114112", "\u{FFFD}"),
        ("56320", "\u{FFFD}"),
    ];
    for (value, expected) in cases {
        assert_eq!(text(&format!("{{{BS}rtf1{BS}uc0 {BS}u{value} }}")), expected, "value {value}");
    }
    assert_eq!(text(&format!("{{{BS}rtf1{BS}uc0 {BS}u55357}}")), "\u{FFFD}");
}

#[test]
fn surrogate_storm() {
    let mut rtf = format!("{{{BS}rtf1{BS}uc0 ");
    for _ in 0..10_000 {
        rtf.push_str(&format!("{BS}u55357 "));
    }
    rtf.push_str(&format!("{BS}u55357 {BS}u56832 }}"));
    let out = timed("surrogate storm", || text(&rtf));
    assert_eq!(out.matches('\u{FFFD}').count(), 10_000);
    assert!(out.ends_with('\u{1F600}'));
}

#[test]
fn parameter_overflow_is_an_error() {
    assert!(outcome(&format!("{{{BS}rtf1 {BS}b99999999999 x}}")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1 {BS}b-99999999999 x}}")).is_err());
    assert_eq!(text(&format!("{{{BS}rtf1 {BS}fi-2147483648 x}}")), "x");
}

#[test]
fn out_of_range_values_are_errors_not_panics() {
    assert!(outcome(&format!("{{{BS}rtf1 {BS}fs-24 x}}")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1 {BS}cf70000 x}}")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1{{{BS}colortbl;{BS}red300{BS}green0{BS}blue0;}}x}}")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1{{{BS}fonttbl{{{BS}f-1 Bad;}}}}x}}")).is_err());
}

#[test]
fn very_long_control_word_and_number() {
    let rtf = format!("{{{BS}rtf1 {BS}{} x}}", "a".repeat(1_000_000));
    assert_eq!(timed("long control word", || text(&rtf)), "x");
    let rtf = format!("{{{BS}rtf1 {BS}b{} x}}", "0".repeat(100_000));
    assert_eq!(timed("long zero parameter", || text(&rtf)), "x");
}

#[test]
fn nul_padding_around_the_document_is_ignored() {
    assert_eq!(text(&format!("{{{BS}rtf1 a}}\u{0}\u{0}\u{0}")), "a");
    assert_eq!(text(&format!("\u{0}\n{{{BS}rtf1 a}}\r\n\u{0}")), "a");
    assert_eq!(RtfDocument::from_bytes(b"{\\rtf1 a}\0\0\0\0").unwrap().get_text(), "a");
}

#[test]
fn nul_and_control_characters_are_text() {
    assert_eq!(text(&format!("{{{BS}rtf1 a\u{0}b\u{7}c\u{1B}d}}")), "a\u{0}b\u{7}c\u{1B}d");
}

#[test]
fn invalid_utf8_input() {
    assert_eq!(RtfDocument::from_bytes(b"{\\rtf1 \xFF\xFE\x80}").unwrap().get_text(), "\u{FF}\u{FE}\u{20AC}");
    assert_eq!(RtfDocument::from_bytes(b"{\\rtf1\\ansicpg0 \xE9}").unwrap().get_text(), "\u{E9}");
    assert_eq!(RtfDocument::from_bytes(b"{\\rtf1\\ansicpg99999999 \x93}").unwrap().get_text(), "\u{201C}");
    assert!(RtfDocument::from_bytes(&[0xFF; 1000]).is_err());
    assert!(RtfDocument::from_bytes(b"").is_err());
}

#[test]
fn malformed_escapes() {
    assert!(outcome(&format!("{{{BS}rtf1 a{BS}")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1 a{BS}'")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1 a{BS}'4")).is_err());
    assert!(outcome(&format!("{{{BS}rtf1 a{BS}'4\u{E9}}}")).is_err());
    assert_eq!(text(&format!("{{{BS}rtf1 a{BS}\u{E9}b}}")), "ab");
    assert_eq!(text(&format!("{{{BS}rtf1 a{BS}{BS}{BS}{{{BS}}}b}}")), "a\\{}b");
}

#[test]
fn code_page_and_charset_abuse() {
    assert_eq!(text(&format!("{{{BS}rtf1{BS}ansi{BS}ansicpg99999999 {BS}'93}}")), "\u{201C}");
    assert_eq!(text(&format!("{{{BS}rtf1{BS}ansi{BS}ansicpg-1252 {BS}'93}}")), "\u{201C}");
    assert_eq!(text(&format!("{{{BS}rtf1{{{BS}fonttbl{{{BS}f0{BS}fcharset999 X;}}}}{BS}f0{BS}'93}}")), "\u{201C}");
    assert_eq!(text(&format!("{{{BS}rtf1{BS}ansicpg0 {BS}'e9}}")), "\u{E9}");
}

#[test]
fn hex_storm() {
    let rtf = format!("{{{BS}rtf1 {}}}", format!("{BS}'41").repeat(200_000));
    assert_eq!(timed("hex storm", || text(&rtf)).len(), 200_000);
}

#[test]
fn large_formatted_text_is_linear() {
    let mut rtf = format!("{{{BS}rtf1 ");
    for i in 0..100_000 {
        rtf.push_str(if i % 2 == 0 { "{\\b word} " } else { "plain " });
    }
    rtf.push('}');
    let doc = timed("large formatted text", || RtfDocument::try_from(rtf.as_str()).unwrap());
    assert_eq!(doc.body.len(), 100_000);
}

#[test]
fn stylesheet_with_deeply_nested_ignorable_groups() {
    let depth = 20_000;
    let nested = format!("{}x{}", format!("{{{BS}*{BS}junk ").repeat(depth), "}".repeat(depth));
    let rtf = format!("{{{BS}rtf1{{{BS}stylesheet{{{BS}s1 {nested} Heading;}}}}body}}");
    assert_eq!(timed("nested stylesheet", || text(&rtf)), "body");
}

#[test]
fn many_ignorable_groups_in_the_font_table() {
    let fonts: String = (0..20_000).map(|i| format!("{{{BS}f{i}{{{BS}*{BS}panose 0202}}Font {i};}}")).collect();
    let rtf = format!("{{{BS}rtf1{{{BS}fonttbl{fonts}}}body}}");
    let doc = timed("font table", || RtfDocument::try_from(rtf.as_str()).unwrap());
    assert_eq!(doc.header.font_table.len(), 20_000);
    assert_eq!(doc.header.font_table.get(&19_999).unwrap().name, "Font 19999");
}

#[test]
fn destinations_out_of_place() {
    assert_eq!(text(&format!("{{{BS}rtf1 a{{{BS}upr{{x}}}}b{{{BS}*{BS}ud{{y}}}}c}}")), "abc");
    assert_eq!(text(&format!("{{{BS}rtf1 a{BS}*b}}")), "ab");
    assert_eq!(text(&format!("{{{BS}rtf1 a{{{BS}*}}b}}")), "ab");
    assert_eq!(text(&format!("{{{BS}rtf1 a{{{BS}* text}}b}}")), "ab");
    let doc = RtfDocument::try_from(format!("{{{BS}rtf1{{{BS}info{{{BS}title A}}}}{{{BS}info{{{BS}title B}}}}x}}").as_str()).unwrap();
    assert_eq!(doc.info.title.as_deref(), Some("B"));
}

#[test]
fn notes_inside_notes_and_headers() {
    let rtf = format!("{{{BS}rtf1 a{BS}chftn{{{BS}footnote{BS}chftn one{{{BS}footnote{BS}chftn two}}{{{BS}header h}}}}b}}");
    let out = text(&rtf);
    assert!(out.starts_with("a1b"), "{out:?}");
    assert!(out.contains("1one") && out.contains("two") && out.ends_with('h'), "{out:?}");
}

#[test]
fn two_byte_code_page_storms() {
    for code_page in ["932", "936", "949", "950"] {
        let storm = format!("{BS}'81A{BS}'fe{BS}{{{BS}'e0{BS}par{BS}'9f{BS}'ff{BS}'81").repeat(20_000);
        let rtf = format!("{{{BS}rtf1{BS}ansi{BS}ansicpg{code_page} {storm}{BS}'81}}");
        timed("lead byte storm", || text(&rtf));
        let mut raw = format!("{{{BS}rtf1{BS}ansi{BS}ansicpg{code_page} ").into_bytes();
        raw.extend((0..200_000usize).map(|i| [0x81u8, 0x5C, 0xFE, 0x7B, 0x7D, 0x41, 0xE0, 0xFF][i % 8]));
        raw.push(b'}');
        let result = timed("raw two-byte storm", || panic::catch_unwind(move || RtfDocument::from_bytes(&raw).map(|d| d.get_text())));
        assert!(result.is_ok(), "panic on raw bytes in code page {code_page}");
    }
}

#[test]
fn every_single_byte_corruption_of_a_two_byte_document() {
    let base = format!("{{{BS}rtf1{BS}ansi{BS}ansicpg932{{{BS}fonttbl{{{BS}f0{BS}fcharset128 {BS}'82{BS}'6c;}}}}{BS}f0 {BS}'93{BS}'fa{BS}'83A{BS}'96{BS}{{{BS}'95{BS}{BS}x}}");
    let bytes = base.as_bytes();
    let replacements: [u8; 8] = [b'{', b'}', b'\\', b'\'', b'A', 0x81, 0xFE, 0xFF];
    for at in 0..bytes.len() {
        for &byte in &replacements {
            let mut mutated = bytes.to_vec();
            mutated[at] = byte;
            let result = panic::catch_unwind(move || RtfDocument::from_bytes(&mutated).map(|d| d.get_text()));
            assert!(result.is_ok(), "panic with byte {byte:#04x} at {at}");
        }
    }
}

#[test]
fn tables_without_structure() {
    assert_eq!(text(&format!("{{{BS}rtf1 {BS}row{BS}row x}}")), "\n\nx");
    assert_eq!(text(&format!("{{{BS}rtf1 {}{BS}row}}", format!("a{BS}cell ").repeat(3))), "a\ta\ta\n");
    assert_eq!(text(&format!("{{{BS}rtf1 a{BS}cell}}")), "a");
}

#[test]
fn every_prefix_of_real_documents() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    for file in ["tests/test-file.rtf", "tests/list-item.rtf", "corpus/word/word-features.rtf", "corpus/libreoffice/international.rtf"] {
        let bytes = fs::read(root.join(file)).expect("resource file");
        let step = (bytes.len() / 3000).max(1);
        for end in (0..=bytes.len()).step_by(step) {
            let prefix = bytes[..end].to_vec();
            let result = panic::catch_unwind(move || RtfDocument::from_bytes(&prefix).map(|d| d.get_text()));
            assert!(result.is_ok(), "panic on the first {end} bytes of {file}");
        }
    }
}

#[test]
fn every_single_byte_corruption_of_a_small_document() {
    let base = format!("{{{BS}rtf1{BS}ansi{BS}ansicpg1252{{{BS}fonttbl{{{BS}f0{BS}fcharset0 A;}}}}{{{BS}info{{{BS}title T}}}}{BS}f0{BS}b x{BS}b0 {BS}'93y{BS}u233 ?{{{BS}footnote{BS}chftn n}}{BS}cell{BS}row}}");
    let bytes = base.as_bytes();
    let replacements: [u8; 12] = [b'{', b'}', b'\\', b'\'', b'*', b'-', b'9', b' ', b'u', 0x00, 0x85, 0xFF];
    for at in 0..bytes.len() {
        for &byte in &replacements {
            let mut mutated = bytes.to_vec();
            mutated[at] = byte;
            let result = panic::catch_unwind(move || RtfDocument::from_bytes(&mutated).map(|d| d.get_text()));
            assert!(result.is_ok(), "panic with byte {byte:#04x} at {at}");
        }
        let mut deleted = bytes.to_vec();
        deleted.remove(at);
        let result = panic::catch_unwind(move || RtfDocument::from_bytes(&deleted).map(|d| d.get_text()));
        assert!(result.is_ok(), "panic with byte {at} deleted");
    }
}
