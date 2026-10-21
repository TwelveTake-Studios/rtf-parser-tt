# rtf-parser-tt

[![Crates.io](https://img.shields.io/crates/v/rtf-parser-tt.svg)](https://crates.io/crates/rtf-parser-tt)
[![Documentation](https://docs.rs/rtf-parser-tt/badge.svg)](https://docs.rs/rtf-parser-tt)
[![License](https://img.shields.io/crates/l/rtf-parser-tt.svg)](LICENSE.md)

An RTF parser that reads the text, formatting and structure of RTF documents. Fork of [rtf-parser](https://crates.io/crates/rtf-parser) by [@d0rianb](https://github.com/d0rianb).

## Why rtf-parser-tt

rtf-parser-tt began as a fork of rtf-parser to preserve special characters such as em dashes and smart quotes. That support was merged into rtf-parser and released in version 0.4.3. rtf-parser-tt is maintained independently, with a focus on reliable handling of real-world documents. Its lexer follows the RTF 1.9.1 specification, and the parser decodes code pages, separates paragraphs and table cells, places footnotes, headers and footers, and reads document properties. Malformed input returns an error instead of causing a panic.

## Support

If you find this crate useful, consider supporting development:

- [Ko-fi](https://ko-fi.com/twelvetake)

## Installation

```toml
[dependencies]
rtf-parser-tt = "0.6"
```

## Usage

```rust
use rtf_parser_tt::RtfDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rtf = r#"{\rtf1\ansi Hello\emdash world}"#;
    let doc = RtfDocument::try_from(rtf)?;
    let text = doc.get_text();
    assert!(text.contains("—"));
    Ok(())
}
```

### Files and code pages

`RtfDocument::from_filepath` and `RtfDocument::from_bytes` accept a document in any code page. UTF-8 input is read as is; any other input is decoded with the code page the document declares.

```rust
use rtf_parser_tt::RtfDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = RtfDocument::from_bytes(b"{\\rtf1\\ansi\\ansicpg1251 \xcf\xf0\xe8\xe2\xe5\xf2}")?;
    assert_eq!(doc.get_text(), "Привет");
    Ok(())
}
```

### Plain text

`get_text` returns the body text followed by footnotes, headers and footers, in the order Word uses when it saves a document as plain text, and omits hidden text. `get_text_with` selects what the text includes.

```rust
use rtf_parser_tt::{RtfDocument, TextOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rtf = r"{\rtf1\ansi Visible{\v  hidden} text{\super\chftn}{\footnote\pard\plain{\super\chftn} A footnote.}\par}";
    let doc = RtfDocument::try_from(rtf)?;
    assert_eq!(doc.get_text(), "Visible text1\n1 A footnote.");
    let options = TextOptions::default().hidden_text(true).notes(false);
    assert_eq!(doc.get_text_with(options), "Visible hidden text\n");
    Ok(())
}
```

### Document structure

`RtfDocument::body` is a list of `StyleBlock` values. Each block holds a run of text with its character formatting (`Painter`), its paragraph properties (`Paragraph`) and the part of the document it belongs to (`Part`).

```rust
use rtf_parser_tt::{Part, RtfDocument};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rtf = r"{\rtf1\ansi Plain {\b bold}{\super\chftn}{\footnote\pard\plain{\super\chftn} A footnote.}\par}";
    let doc = RtfDocument::try_from(rtf)?;
    let bold: Vec<&str> = doc.body.iter().filter(|block| block.painter.bold).map(|block| block.text.as_str()).collect();
    assert_eq!(bold, ["bold"]);
    let note: String = doc.body.iter().filter(|block| block.part == Part::Note(1)).map(|block| block.text.as_str()).collect();
    assert_eq!(note, "1 A footnote.");
    Ok(())
}
```

## Special Characters and Breaks

| Control Word | Unicode | Character |
|--------------|---------|-----------|
| `\emdash` | U+2014 | — |
| `\endash` | U+2013 | – |
| `\bullet` | U+2022 | • |
| `\lquote` | U+2018 | ‘ |
| `\rquote` | U+2019 | ’ |
| `\ldblquote` | U+201C | “ |
| `\rdblquote` | U+201D | ” |
| `\~` | U+00A0 | (no-break space) |
| `\-` | U+00AD | (soft hyphen) |
| `\_` | U+2011 | (non-breaking hyphen) |
| `\tab`, `\cell` | U+0009 | (tab) |
| `\line`, `\par`, `\sect`, `\page`, `\row` | U+000A | (newline) |

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `encoding_rs` | Enabled | Decodes the Japanese (932), Simplified Chinese (936), Korean (949) and Traditional Chinese (950) code pages through [encoding_rs](https://crates.io/crates/encoding_rs). Without it, these bytes are read as Latin-1. |
| `jsbindings` | Disabled | WebAssembly bindings through `wasm-bindgen`. |

To build without `encoding_rs`:

```toml
[dependencies]
rtf-parser-tt = { version = "0.6", default-features = false }
```

## API

Key types:
- `RtfDocument` - Parsed document: header, document properties and body
- `StyleBlock` - A run of text with its formatting, paragraph properties and part of the document
- `Part` - Body text, footnote reference, footnote, header or footer
- `TextOptions` - What `get_text_with` includes
- `Painter` - Character formatting (bold, italic, hidden, etc.)
- `Paragraph` - Paragraph properties (alignment, spacing, indents)
- `DocumentInfo` - Title, author and other document properties
- `RtfError` - Error returned for malformed input
- `Lexer` - Tokenizes RTF input
- `Parser` - Converts tokens to a document

Changes from 0.5 that affect existing code are listed in the [changelog](https://github.com/TwelveTake-Studios/rtf-parser-tt/blob/master/CHANGELOG.md).

## License

MIT License - same as upstream.

## Credits

- Original crate: [rtf-parser](https://github.com/d0rianb/rtf-parser) by [@d0rianb](https://github.com/d0rianb)
- Fork maintained by [TwelveTake Studios](https://twelvetake.com)
