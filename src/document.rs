use std::fmt;
use std::fs;
use std::io::Read;

use serde::{Deserialize, Serialize};
#[cfg(feature = "jsbindings")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::encoding::decode_document_bytes;
use crate::header::RtfHeader;
use crate::lexer::{Lexer, LexerError};
use crate::parser::{Parser, ParserError, Part, StyleBlock};

#[cfg_attr(feature = "jsbindings", wasm_bindgen)]
pub fn parse_rtf(rtf: String) -> Result<RtfDocument, RtfError> {
    RtfDocument::try_from(rtf)
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RtfError {
    Lexer(LexerError),
    Parser(ParserError),
    Io(String),
}

impl std::error::Error for RtfError {}

impl fmt::Display for RtfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RtfError::Lexer(error) => write!(f, "{error}"),
            RtfError::Parser(error) => write!(f, "{error}"),
            RtfError::Io(error) => write!(f, "[RTF] : {error}"),
        }
    }
}

impl From<LexerError> for RtfError {
    fn from(error: LexerError) -> Self {
        RtfError::Lexer(error)
    }
}

impl From<ParserError> for RtfError {
    fn from(error: ParserError) -> Self {
        RtfError::Parser(error)
    }
}

impl From<std::io::Error> for RtfError {
    fn from(error: std::io::Error) -> Self {
        RtfError::Io(error.to_string())
    }
}

#[cfg(feature = "jsbindings")]
impl From<RtfError> for wasm_bindgen::JsValue {
    fn from(error: RtfError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "jsbindings", wasm_bindgen(getter_with_clone))]
#[non_exhaustive]
pub struct DocumentInfo {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub author: Option<String>,
    pub manager: Option<String>,
    pub company: Option<String>,
    pub operator: Option<String>,
    pub category: Option<String>,
    pub keywords: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TextOptions {
    pub hidden_text: bool,
    pub notes: bool,
    pub headers_footers: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            hidden_text: false,
            notes: true,
            headers_footers: true,
        }
    }
}

impl TextOptions {
    #[must_use]
    pub fn hidden_text(mut self, include: bool) -> Self {
        self.hidden_text = include;
        self
    }

    #[must_use]
    pub fn notes(mut self, include: bool) -> Self {
        self.notes = include;
        self
    }

    #[must_use]
    pub fn headers_footers(mut self, include: bool) -> Self {
        self.headers_footers = include;
        self
    }

    pub fn includes(&self, block: &StyleBlock) -> bool {
        let part = match block.part {
            Part::Body => true,
            Part::NoteReference(_) | Part::Note(_) => self.notes,
            Part::Header | Part::Footer => self.headers_footers,
        };
        part && (self.hidden_text || !block.painter.hidden)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "jsbindings", wasm_bindgen(getter_with_clone))]
#[non_exhaustive]
pub struct RtfDocument {
    pub header: RtfHeader,
    pub info: DocumentInfo,
    pub body: Vec<StyleBlock>,
}

impl TryFrom<String> for RtfDocument {
    type Error = RtfError;
    fn try_from(file_content: String) -> Result<Self, Self::Error> {
        Self::try_from(file_content.as_str())
    }
}

impl TryFrom<&str> for RtfDocument {
    type Error = RtfError;
    fn try_from(file_content: &str) -> Result<Self, Self::Error> {
        let tokens = Lexer::scan(file_content)?;
        Ok(Parser::new(tokens).parse()?)
    }
}

impl TryFrom<&mut fs::File> for RtfDocument {
    type Error = RtfError;
    fn try_from(file: &mut fs::File) -> Result<Self, Self::Error> {
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }
}

impl RtfDocument {
    /// Create an `RtfDocument` from a rtf file path
    pub fn from_filepath(filename: &str) -> Result<RtfDocument, RtfError> {
        Self::from_bytes(&fs::read(filename)?)
    }

    /// Create an `RtfDocument` from the bytes of an RTF file. UTF-8 input is read as is; any other
    /// input is read in the code page the document declares.
    pub fn from_bytes(bytes: &[u8]) -> Result<RtfDocument, RtfError> {
        match std::str::from_utf8(bytes) {
            Ok(text) => Self::try_from(text),
            Err(_) => Self::try_from(decode_document_bytes(bytes).as_str()),
        }
    }

    /// Get the raw text of an RTF document
    pub fn get_text(&self) -> String {
        self.get_text_with(TextOptions::default())
    }

    pub fn get_text_with(&self, options: TextOptions) -> String {
        self.body.iter().filter(|block| options.includes(block)).map(|block| block.text.as_str()).collect()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::document::RtfDocument;

    #[test]
    fn get_text_from_document() {
        let rtf = r#"{ \rtf1\ansi{\fonttbl\f0\fswiss Helvetica;}\f0\pard Voici du texte en {\b gras}.\par }"#;
        let document = RtfDocument::try_from(rtf).unwrap();
        assert_eq!(document.get_text(), "Voici du texte en gras.\n")
    }

    #[test]
    fn create_document_from_file() {
        let mut file = fs::File::open("./resources/tests/test-file.rtf").unwrap();
        let document = RtfDocument::try_from(&mut file).unwrap();
        assert_eq!(document.header.font_table.get(&0).unwrap().name, String::from("Helvetica"));
    }

    #[test]
    fn create_document_from_filepath() {
        let filename = "./resources/tests/test-file.rtf";
        let document = RtfDocument::from_filepath(filename).unwrap();
        assert_eq!(document.header.font_table.get(&0).unwrap().name, String::from("Helvetica"));
    }

    #[test]
    fn create_document_from_code_page_bytes() {
        let document = RtfDocument::from_bytes(b"{\\rtf1\\ansi\\ansicpg1251 \xCF\xF0\xE8\xE2\xE5\xF2}").unwrap();
        assert_eq!(document.get_text(), "\u{41F}\u{440}\u{438}\u{432}\u{435}\u{442}");
    }

    #[test]
    fn text_options_choose_what_get_text_includes() {
        let rtf = r"{\rtf1\ansi{\header Head\par}Shown{\v Hidden} body{\super\chftn}{\footnote\pard\plain{\super\chftn} Note.}\par}";
        let doc = RtfDocument::try_from(rtf).unwrap();
        let only = |hidden_text, notes, headers_footers| doc.get_text_with(TextOptions::default().hidden_text(hidden_text).notes(notes).headers_footers(headers_footers));
        assert_eq!(doc.get_text(), "Shown body1\n1 Note.\nHead\n");
        assert_eq!(only(true, true, true), "ShownHidden body1\n1 Note.\nHead\n");
        assert_eq!(only(false, false, true), "Shown body\nHead\n");
        assert_eq!(only(false, true, false), "Shown body1\n1 Note.\n");
        assert_eq!(only(false, false, false), "Shown body\n");
    }

    #[test]
    fn leaving_out_notes_keeps_the_body_line_break() {
        let doc = RtfDocument::try_from(r"{\rtf1\ansi Body{\super\chftn}{\footnote\pard\plain{\super\chftn} N.}}").unwrap();
        assert_eq!(doc.get_text(), "Body1\n1 N.");
        assert_eq!(doc.get_text_with(TextOptions::default().notes(false)), "Body\n");
    }

    #[test]
    fn default_text_options() {
        assert_eq!(
            TextOptions::default(),
            TextOptions {
                hidden_text: false,
                notes: true,
                headers_footers: true
            }
        );
    }

    #[test]
    fn parse_rtf_returns_an_error_instead_of_panicking() {
        assert!(parse_rtf(r"{\rtf1 Hello \'zz world}".to_string()).is_err());
        assert_eq!(parse_rtf(r"{\rtf1 Hello}".to_string()).unwrap().get_text(), "Hello");
    }
}
