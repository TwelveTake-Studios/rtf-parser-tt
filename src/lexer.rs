use std::fmt;

use crate::tokens::{ControlWord, Property, Token};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LexerError {
    Error(String),
    InvalidUnicode(String),
    InvalidLastChar,
}

impl std::error::Error for LexerError {}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[RTF Lexer] : ")?;
        match self {
            LexerError::InvalidLastChar => write!(f, "Invalid last char, should be '}}'"),
            LexerError::InvalidUnicode(uc) => write!(f, "Invalid unicode : {uc}"),
            LexerError::Error(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<std::str::Utf8Error> for LexerError {
    fn from(value: std::str::Utf8Error) -> Self {
        LexerError::Error(value.to_string())
    }
}

impl From<std::num::ParseIntError> for LexerError {
    fn from(value: std::num::ParseIntError) -> Self {
        LexerError::Error(value.to_string())
    }
}

const fn special_bytes() -> [bool; 256] {
    let mut table = [false; 256];
    table[b'{' as usize] = true;
    table[b'}' as usize] = true;
    table[b'\\' as usize] = true;
    table[b'\r' as usize] = true;
    table[b'\n' as usize] = true;
    table
}

static SPECIAL: [bool; 256] = special_bytes();

pub struct Lexer;

impl Lexer {
    pub fn scan(src: &str) -> Result<Vec<Token<'_>>, LexerError> {
        let src = src.trim_matches(|c: char| c.is_whitespace() || c == '\0');
        let bytes = src.as_bytes();
        let len = bytes.len();
        let mut tokens = Vec::with_capacity(len / 32 + 1);
        let mut text_start = 0;
        let mut i = 0;
        while let Some(offset) = bytes[i..].iter().position(|&b| SPECIAL[usize::from(b)]) {
            i += offset;
            if text_start < i {
                tokens.push(Token::PlainText(&src[text_start..i]));
            }
            i = match bytes[i] {
                b'{' => {
                    tokens.push(Token::OpeningBracket);
                    i + 1
                }
                b'}' => {
                    tokens.push(Token::ClosingBracket);
                    i + 1
                }
                b'\\' => Self::escape(src, i, &mut tokens)?,
                _ => i + 1,
            };
            text_start = i;
        }
        if text_start < len {
            tokens.push(Token::PlainText(&src[text_start..len]));
        }
        Ok(tokens)
    }

    fn escape<'a>(src: &'a str, start: usize, tokens: &mut Vec<Token<'a>>) -> Result<usize, LexerError> {
        let bytes = src.as_bytes();
        let Some(&next) = bytes.get(start + 1) else {
            return Ok(start + 1);
        };
        let symbol = |word| Token::ControlSymbol((word, Property::None));
        match next {
            b'a'..=b'z' | b'A'..=b'Z' => Self::control_word(src, start, tokens),
            b'\'' => {
                let hex = src.get(start + 2..start + 4).filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()));
                let Some(hex) = hex else {
                    let found: String = src[start + 1..].chars().take(3).collect();
                    return Err(LexerError::InvalidUnicode(found));
                };
                let byte = u8::from_str_radix(hex, 16)?;
                tokens.push(Token::ControlSymbol((ControlWord::HexChar, Property::Value(byte as i32))));
                Ok(start + 4)
            }
            b'{' | b'}' | b'\\' => {
                tokens.push(Token::PlainText(&src[start + 1..start + 2]));
                Ok(start + 2)
            }
            b'\n' => {
                tokens.push(Token::CRLF);
                Ok(start + 2)
            }
            b'\r' => {
                tokens.push(Token::CRLF);
                Ok(if bytes.get(start + 2) == Some(&b'\n') { start + 3 } else { start + 2 })
            }
            b'*' => {
                tokens.push(Token::IgnorableDestination);
                Ok(start + 2)
            }
            b'~' | b'-' | b'_' | b'|' | b':' | b'\t' => {
                tokens.push(symbol(match next {
                    b'~' => ControlWord::NonBreakingSpace,
                    b'-' => ControlWord::OptionalHyphen,
                    b'_' => ControlWord::NonBreakingHyphen,
                    b'|' => ControlWord::FormulaCharacter,
                    b':' => ControlWord::IndexSubentry,
                    _ => ControlWord::Tab,
                }));
                Ok(start + 2)
            }
            _ => Ok(start + 1 + src[start + 1..].chars().next().map_or(1, char::len_utf8)),
        }
    }

    fn control_word<'a>(src: &'a str, start: usize, tokens: &mut Vec<Token<'a>>) -> Result<usize, LexerError> {
        let bytes = src.as_bytes();
        let len = bytes.len();
        let mut end = start + 1;
        while end < len && bytes[end].is_ascii_alphabetic() {
            end += 1;
        }
        let digits = if end + 1 < len && bytes[end] == b'-' && bytes[end + 1].is_ascii_digit() { end + 1 } else { end };
        let mut param_end = digits;
        while param_end < len && bytes[param_end].is_ascii_digit() {
            param_end += 1;
        }
        if param_end > digits {
            end = param_end;
        }
        let symbol = ControlWord::from(&src[start..end])?;
        tokens.push(Token::ControlSymbol(symbol));
        let mut next = if bytes.get(end) == Some(&b' ') { end + 1 } else { end };
        if let (ControlWord::Bin, Property::Value(count)) = symbol {
            if count > 0 {
                next = next.saturating_add(count as usize).min(len);
                while !src.is_char_boundary(next) {
                    next += 1;
                }
            }
        }
        Ok(next)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::lexer::Lexer;
    use crate::tokens::ControlWord::{
        Ansi, Bold, ColorBlue, ColorNumber, ColorRed, FontNumber, FontSize, FontTable, HexChar, Italic, NonBreakingHyphen, NonBreakingSpace,
        OptionalHyphen, Par, Pard, Rtf, Underline, Unknown,
    };
    use crate::tokens::Property::*;
    use crate::tokens::Token::*;

    #[test]
    fn scan_entire_file_test() {
        let tokens = Lexer::scan(r#"{ \rtf1\ansi{\fonttbl\f0\fswiss Helvetica;}\f0\pard Voici du texte en {\b gras}.\par }"#);
        assert_eq!(
            tokens.unwrap(),
            vec![
                OpeningBracket,
                PlainText(" "),
                ControlSymbol((Rtf, Value(1))),
                ControlSymbol((Ansi, None)),
                OpeningBracket,
                ControlSymbol((FontTable, None)),
                ControlSymbol((FontNumber, Value(0))),
                ControlSymbol((Unknown("\\fswiss"), None)),
                PlainText("Helvetica;"),
                ClosingBracket,
                ControlSymbol((FontNumber, Value(0))),
                ControlSymbol((Pard, None)),
                PlainText("Voici du texte en "),
                OpeningBracket,
                ControlSymbol((Bold, None)),
                PlainText("gras"),
                ClosingBracket,
                PlainText("."),
                ControlSymbol((Par, None)),
                ClosingBracket,
            ]
        );
    }

    #[test]
    fn scan_escaped_text() {
        let tokens = Lexer::scan(
            r#"\f0\fs24 \cf0 test de code \
if (a == b) \{\
    test();\
\} else \{\
    return;\
\}}"#,
        );
        assert_eq!(
            tokens.unwrap(),
            vec![
                ControlSymbol((FontNumber, Value(0))),
                ControlSymbol((FontSize, Value(24))),
                ControlSymbol((ColorNumber, Value(0))),
                PlainText("test de code "),
                CRLF,
                PlainText("if (a == b) "),
                PlainText("{"),
                CRLF,
                PlainText("    test();"),
                CRLF,
                PlainText("}"),
                PlainText(" else "),
                PlainText("{"),
                CRLF,
                PlainText("    return;"),
                CRLF,
                PlainText("}"),
                ClosingBracket
            ],
        );
    }

    #[test]
    fn scan_ignorable_destination() {
        let tokens = Lexer::scan(r"{\*\expandedcolortbl;;}");
        assert_eq!(
            tokens.unwrap(),
            vec![OpeningBracket, IgnorableDestination, ControlSymbol((Unknown(r"\expandedcolortbl"), None)), PlainText(";;"), ClosingBracket]
        )
    }

    #[test]
    fn semicolon_after_control_word_is_text() {
        let tokens = Lexer::scan(r"{\red255\blue255;}");
        assert_eq!(
            tokens.unwrap(),
            vec![OpeningBracket, ControlSymbol((ColorRed, Value(255))), ControlSymbol((ColorBlue, Value(255))), PlainText(";"), ClosingBracket]
        );
    }

    #[test]
    fn lex_with_leading_whitespaces() {
        let tokens = Lexer::scan("\t {\\rtf1 }\n ").unwrap();
        assert_eq!(tokens, vec![OpeningBracket, ControlSymbol((Rtf, Value(1))), ClosingBracket]);
    }

    #[test]
    fn unescaped_line_breaks_are_ignored() {
        let text = r#"{\partightenfactor0

\fs24 \cf0 Font size 12,
\f0\b bold text. \ul Underline,bold text.\
 }"#;
        let tokens = Lexer::scan(text).unwrap();
        assert_eq!(
            tokens,
            [
                OpeningBracket,
                ControlSymbol((Unknown("\\partightenfactor"), Value(0))),
                ControlSymbol((FontSize, Value(24))),
                ControlSymbol((ColorNumber, Value(0))),
                PlainText("Font size 12,"),
                ControlSymbol((FontNumber, Value(0))),
                ControlSymbol((Bold, None)),
                PlainText("bold text. "),
                ControlSymbol((Underline, None)),
                PlainText("Underline,bold text."),
                CRLF,
                PlainText(" "),
                ClosingBracket
            ]
        );
    }

    #[test]
    fn space_after_control_word() {
        let tokens = Lexer::scan(r"{in{\i cred}ible}").unwrap();
        assert_eq!(
            tokens,
            [OpeningBracket, PlainText("in"), OpeningBracket, ControlSymbol((Italic, None)), PlainText("cred"), ClosingBracket, PlainText("ible"), ClosingBracket,]
        )
    }

    #[test]
    fn hex_escape_is_its_own_token() {
        let tokens = Lexer::scan(r"{je suis une b\'eate}").unwrap();
        assert_eq!(tokens, [OpeningBracket, PlainText("je suis une b"), ControlSymbol((HexChar, Value(234))), PlainText("te"), ClosingBracket]);
    }

    #[test]
    fn punctuation_ends_a_control_word() {
        let tokens = Lexer::scan(r"{\b Bold\b0, then}").unwrap();
        assert_eq!(
            tokens,
            [OpeningBracket, ControlSymbol((Bold, None)), PlainText("Bold"), ControlSymbol((Bold, Value(0))), PlainText(", then"), ClosingBracket]
        );
    }

    #[test]
    fn only_one_space_is_a_delimiter() {
        let tokens = Lexer::scan(r"{\b  two}").unwrap();
        assert_eq!(tokens, [OpeningBracket, ControlSymbol((Bold, None)), PlainText(" two"), ClosingBracket]);
    }

    #[test]
    fn non_ascii_ends_a_control_word() {
        let tokens = Lexer::scan("{\\i\u{420}\u{438}\u{43C}}").unwrap();
        assert_eq!(tokens, [OpeningBracket, ControlSymbol((Italic, None)), PlainText("\u{420}\u{438}\u{43C}"), ClosingBracket]);
    }

    #[test]
    fn control_symbols() {
        let tokens = Lexer::scan(r"{A\~B\-C\_D}").unwrap();
        assert_eq!(
            tokens,
            [
                OpeningBracket,
                PlainText("A"),
                ControlSymbol((NonBreakingSpace, None)),
                PlainText("B"),
                ControlSymbol((OptionalHyphen, None)),
                PlainText("C"),
                ControlSymbol((NonBreakingHyphen, None)),
                PlainText("D"),
                ClosingBracket
            ]
        );
    }

    #[test]
    fn negative_parameter_needs_a_digit() {
        let tokens = Lexer::scan(r"{\fi-180 x\b- y}").unwrap();
        assert_eq!(
            tokens,
            [
                OpeningBracket,
                ControlSymbol((crate::tokens::ControlWord::FirstLineIdent, Value(-180))),
                PlainText("x"),
                ControlSymbol((Bold, None)),
                PlainText("- y"),
                ClosingBracket
            ]
        );
    }

    #[test]
    fn bin_data_is_skipped() {
        let tokens = Lexer::scan("{a\\bin5 }{\\x}b}").unwrap();
        assert_eq!(tokens, [OpeningBracket, PlainText("a"), ControlSymbol((crate::tokens::ControlWord::Bin, Value(5))), PlainText("b"), ClosingBracket]);
    }

    #[test]
    fn bin_data_stops_at_the_end() {
        let tokens = Lexer::scan("{a\\bin99 xy").unwrap();
        assert_eq!(tokens, [OpeningBracket, PlainText("a"), ControlSymbol((crate::tokens::ControlWord::Bin, Value(99)))]);
    }

    #[test]
    fn unknown_control_symbol_is_dropped_whole() {
        let tokens = Lexer::scan("{a\\\u{420}b}").unwrap();
        assert_eq!(tokens, [OpeningBracket, PlainText("a"), PlainText("b"), ClosingBracket]);
    }

    #[test]
    fn truncated_hex_escape_is_an_error() {
        assert!(Lexer::scan(r"{\'5}").is_err());
        assert!(Lexer::scan(r"{\'zz}").is_err());
        assert!(Lexer::scan("{\\'").is_err());
    }
}
