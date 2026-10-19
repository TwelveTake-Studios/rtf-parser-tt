# rtf-parser-tt

[![Crates.io](https://img.shields.io/crates/v/rtf-parser-tt.svg)](https://crates.io/crates/rtf-parser-tt)
[![Documentation](https://docs.rs/rtf-parser-tt/badge.svg)](https://docs.rs/rtf-parser-tt)
[![License](https://img.shields.io/crates/l/rtf-parser-tt.svg)](LICENSE.md)

RTF parser with special character support. Fork of [rtf-parser](https://crates.io/crates/rtf-parser) by [@d0rianb](https://github.com/d0rianb).

## Why rtf-parser-tt

rtf-parser-tt began as a fork of rtf-parser to preserve special characters such as em dashes and smart quotes. That support was merged into rtf-parser and released in version 0.4.3. rtf-parser-tt is maintained independently, with a focus on reliable handling of real-world documents: malformed input returns an error instead of causing a panic.

## Support

If you find this crate useful, consider supporting development:

- [GitHub Sponsors](https://github.com/sponsors/twelvetake)
- [Ko-fi](https://ko-fi.com/twelvetake)

## Installation

```toml
[dependencies]
rtf-parser-tt = "0.5"
```

## What's Added

| Control Word | Unicode | Character |
|--------------|---------|-----------|
| `\emdash` | U+2014 | — |
| `\endash` | U+2013 | – |
| `\bullet` | U+2022 | • |
| `\lquote` | U+2018 | ' |
| `\rquote` | U+2019 | ' |
| `\ldblquote` | U+201C | " |
| `\rdblquote` | U+201D | " |
| `\tab` | U+0009 | (tab) |
| `\line` | U+000A | (newline) |

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

## API

The API is identical to `rtf-parser`. See the [original documentation](https://docs.rs/rtf-parser) for full details.

Key types:
- `RtfDocument` - Parsed RTF document
- `Lexer` - Tokenizes RTF input
- `Parser` - Converts tokens to document
- `StyleBlock` - Text with formatting info
- `Painter` - Text style (bold, italic, etc.)
- `Paragraph` - Layout info (alignment, spacing)

## License

MIT License - same as upstream.

## Credits

- Original crate: [rtf-parser](https://github.com/d0rianb/rtf-parser) by [@d0rianb](https://github.com/d0rianb)
- Fork maintained by [TwelveTake Studios](https://twelvetake.com)
