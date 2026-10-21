use crate::codepages::{single_byte_table, SYMBOL_FONT};
use crate::header::{CharacterSet, Font};

pub(crate) const DEFAULT_CODE_PAGE: u16 = 1252;
pub(crate) const SYMBOL_CODE_PAGE: u16 = 42;

pub(crate) fn code_page_for_font(font: &Font) -> Option<u16> {
    if font.character_set == 2 && font.name.eq_ignore_ascii_case("Symbol") {
        return Some(SYMBOL_CODE_PAGE);
    }
    code_page_for_charset(font.character_set)
}

pub(crate) fn symbol_font_char(byte: u8) -> Option<char> {
    let mapped = SYMBOL_FONT.get(usize::from(byte).checked_sub(0x20)?)?;
    char::from_u32(u32::from(*mapped))
}

pub(crate) fn code_page_for_charset(charset: u8) -> Option<u16> {
    match charset {
        77 => Some(10000),
        128 => Some(932),
        129 => Some(949),
        130 => Some(1361),
        134 => Some(936),
        136 => Some(950),
        161 => Some(1253),
        162 => Some(1254),
        163 => Some(1258),
        177 => Some(1255),
        178 => Some(1256),
        186 => Some(1257),
        204 => Some(1251),
        222 => Some(874),
        238 => Some(1250),
        254 => Some(437),
        255 => Some(850),
        _ => None,
    }
}

pub(crate) fn code_page_for_document(character_set: &CharacterSet) -> u16 {
    match character_set {
        CharacterSet::Ansi => DEFAULT_CODE_PAGE,
        CharacterSet::Mac => 10000,
        CharacterSet::Pc => 437,
        CharacterSet::Pca => 850,
        CharacterSet::Ansicpg(code_page) => *code_page,
    }
}

#[cfg(feature = "encoding_rs")]
fn multi_byte_encoding(code_page: u16) -> Option<&'static encoding_rs::Encoding> {
    match code_page {
        932 => Some(encoding_rs::SHIFT_JIS),
        936 => Some(encoding_rs::GBK),
        949 => Some(encoding_rs::EUC_KR),
        950 => Some(encoding_rs::BIG5),
        _ => None,
    }
}

#[cfg(feature = "encoding_rs")]
pub(crate) fn needs_trail_byte(code_page: u16, bytes: &[u8]) -> bool {
    let lead = |byte: u8| match code_page {
        932 => matches!(byte, 0x81..=0x9F | 0xE0..=0xFC),
        936 | 949 | 950 => matches!(byte, 0x81..=0xFE),
        _ => false,
    };
    let mut index = 0;
    while index < bytes.len() {
        index += if lead(bytes[index]) { 2 } else { 1 };
    }
    index > bytes.len()
}

#[cfg(not(feature = "encoding_rs"))]
pub(crate) fn needs_trail_byte(_code_page: u16, _bytes: &[u8]) -> bool {
    false
}

pub(crate) fn decode_into(code_page: u16, bytes: &[u8], out: &mut String) {
    if code_page == 65001 {
        out.push_str(&String::from_utf8_lossy(bytes));
        return;
    }
    #[cfg(feature = "encoding_rs")]
    if let Some(encoding) = multi_byte_encoding(code_page) {
        out.push_str(&encoding.decode_without_bom_handling(bytes).0);
        return;
    }
    if code_page == SYMBOL_CODE_PAGE {
        out.extend(bytes.iter().map(|&byte| symbol_font_char(byte).unwrap_or(byte as char)));
        return;
    }
    let table = single_byte_table(code_page);
    for &byte in bytes {
        match (byte, table) {
            (0..=0x7F, _) => out.push(byte as char),
            (_, Some(table)) => out.push(char::from_u32(table[(byte - 0x80) as usize] as u32).unwrap_or('\u{FFFD}')),
            (_, None) => out.push(byte as char),
        }
    }
}

pub(crate) fn decode_document_bytes(bytes: &[u8]) -> String {
    let code_page = document_code_page(bytes);
    let mut out = String::with_capacity(bytes.len());
    let mut run_start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && bytes[i + 1..].starts_with(b"bin") {
            let digits_start = i + 4;
            let digits_end = digits_start + bytes[digits_start..].iter().take_while(|b| b.is_ascii_digit()).count();
            if digits_end > digits_start {
                decode_into(code_page, &bytes[run_start..i], &mut out);
                out.push_str("\\bin0 ");
                let count: usize = std::str::from_utf8(&bytes[digits_start..digits_end]).ok().and_then(|n| n.parse().ok()).unwrap_or(0);
                let payload = if bytes.get(digits_end) == Some(&b' ') { digits_end + 1 } else { digits_end };
                i = payload.saturating_add(count).min(bytes.len());
                run_start = i;
                continue;
            }
        }
        i += 1;
    }
    decode_into(code_page, &bytes[run_start..], &mut out);
    out
}

fn document_code_page(bytes: &[u8]) -> u16 {
    let header = &bytes[..bytes.len().min(4096)];
    if let Some(at) = find(header, b"\\ansicpg") {
        let digits: String = header[at + 8..].iter().take_while(|b| b.is_ascii_digit()).map(|&b| b as char).collect();
        if let Ok(code_page) = digits.parse() {
            return code_page;
        }
    }
    let word = |name: &[u8]| {
        find(header, name).is_some_and(|at| !matches!(header.get(at + name.len()), Some(b) if b.is_ascii_alphanumeric()))
    };
    if word(b"\\mac") {
        10000
    } else if word(b"\\pca") {
        850
    } else if word(b"\\pc") {
        437
    } else {
        DEFAULT_CODE_PAGE
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp1252_punctuation() {
        let mut out = String::new();
        decode_into(1252, &[0x93, b'H', 0x94, 0x96, 0x97, 0x85, 0x80], &mut out);
        assert_eq!(out, "\u{201C}H\u{201D}\u{2013}\u{2014}\u{2026}\u{20AC}");
    }

    #[test]
    fn cp1251_cyrillic() {
        let mut out = String::new();
        decode_into(1251, &[0xCF, 0xF0, 0xE8], &mut out);
        assert_eq!(out, "\u{41F}\u{440}\u{438}");
    }

    #[test]
    fn undefined_byte_is_replacement_character() {
        let mut out = String::new();
        decode_into(1253, &[0xAA], &mut out);
        assert_eq!(out, "\u{FFFD}");
    }

    #[test]
    fn symbol_font_bytes() {
        let mut out = String::new();
        decode_into(SYMBOL_CODE_PAGE, &[0xB7, b'a', b'p', 0xB0], &mut out);
        assert_eq!(out, "\u{2022}\u{3B1}\u{3C0}\u{B0}");
    }

    #[test]
    fn utf8_code_page() {
        let mut out = String::new();
        decode_into(65001, "\u{E9}".as_bytes(), &mut out);
        assert_eq!(out, "\u{E9}");
    }

    #[test]
    fn raw_bytes_use_the_declared_code_page() {
        assert_eq!(decode_document_bytes(b"{\\rtf1\\ansi\\ansicpg1251 \xCF\xF0}"), "{\\rtf1\\ansi\\ansicpg1251 \u{41F}\u{440}}");
        assert_eq!(decode_document_bytes(b"{\\rtf1\\mac \xD0}"), "{\\rtf1\\mac \u{2013}}");
        assert_eq!(decode_document_bytes(b"{\\rtf1 \x93}"), "{\\rtf1 \u{201C}}");
    }

    #[test]
    fn raw_bytes_drop_binary_payloads() {
        assert_eq!(decode_document_bytes(b"{a{\\pict\\bin3 \xFF}\xFE}b}"), "{a{\\pict\\bin0 }b}");
        assert_eq!(decode_document_bytes(b"{a\\bin9 \xFF"), "{a\\bin0 ");
    }
}
