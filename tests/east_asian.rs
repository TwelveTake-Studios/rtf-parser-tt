use rtf_parser_tt::RtfDocument;

fn text(rtf: &str) -> String {
    RtfDocument::try_from(rtf).expect("document should parse").get_text()
}

#[cfg(feature = "encoding_rs")]
mod with_encoding_rs {
    use super::*;

    #[test]
    fn shift_jis_pairs() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'93\'fa\'96\'7b\'8c\'ea}"), "\u{65e5}\u{672c}\u{8a9e}");
    }

    #[test]
    fn gbk_pairs() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg936 \'d6\'d0\'ce\'c4}"), "\u{4e2d}\u{6587}");
    }

    #[test]
    fn korean_pairs() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg949 \'c7\'d1\'b1\'b9\'be\'ee}"), "\u{d55c}\u{ad6d}\u{c5b4}");
    }

    #[test]
    fn big5_pairs() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg950 \'a4\'a4\'a4\'e5}"), "\u{4e2d}\u{6587}");
    }

    #[test]
    fn second_byte_written_as_a_letter() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'83A and more}"), "\u{30a2} and more");
    }

    #[test]
    fn second_byte_written_as_an_escaped_brace() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'93\'fa\'96\{\'8c\'ea}"), "\u{65e5}\u{672c}\u{8a9e}");
    }

    #[test]
    fn second_byte_written_as_an_escaped_backslash() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'95\\ end}"), "\u{8868} end");
    }

    #[test]
    fn lone_first_byte_is_a_replacement_character() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'93\par x}"), "\u{fffd}\nx");
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'93 x}"), "\u{fffd} x");
    }

    #[test]
    fn single_byte_katakana() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'b1\'b2}"), "\u{ff71}\u{ff72}");
    }

    #[test]
    fn font_charset_sets_the_code_page() {
        let rtf = r"{\rtf1\ansi\ansicpg1252{\fonttbl{\f0 Arial;}{\f1\fcharset128 MS Mincho;}}\f0 A {\f1 \'93\'fa} B \'e9}";
        assert_eq!(text(rtf), "A \u{65e5} B \u{e9}");
    }

    #[test]
    fn font_name_in_shift_jis() {
        let doc = RtfDocument::try_from(r"{\rtf1\ansi{\fonttbl{\f0\fnil\fcharset128 \'82\'6c\'82\'72 \'96\'be\'92\'a9;}}\f0 x}").unwrap();
        assert_eq!(doc.header.font_table[&0].name, "\u{ff2d}\u{ff33} \u{660e}\u{671d}");
    }

    #[test]
    fn unicode_with_two_byte_fallback() {
        let rtf = concat!(r"{\rtf1\ansi\ansicpg932\uc2", r"\u", "26085", r"\'93\'fa x}");
        assert_eq!(text(rtf), "\u{65e5} x");
    }

    #[test]
    fn raw_shift_jis_bytes() {
        let mut bytes = br"{\rtf1\ansi\ansicpg932 ".to_vec();
        bytes.extend_from_slice(&[0x93, 0xfa, 0x96, 0x7b, 0x95, 0x5c, b'}']);
        assert_eq!(RtfDocument::from_bytes(&bytes).unwrap().get_text(), "\u{65e5}\u{672c}\u{8868}");
    }

    #[test]
    fn single_byte_code_pages_are_unchanged() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg1251 \'cf\'f0\'e8}"), "\u{41f}\u{440}\u{438}");
        assert_eq!(text(r"{\rtf1\ansi\ansicpg1252 \'e9A}"), "\u{e9}A");
    }
}

#[cfg(not(feature = "encoding_rs"))]
mod without_encoding_rs {
    use super::*;

    #[test]
    fn two_byte_code_pages_stay_latin_1() {
        assert_eq!(text(r"{\rtf1\ansi\ansicpg932 \'93\'fa\'83A}"), "\u{93}\u{fa}\u{83}A");
    }
}
