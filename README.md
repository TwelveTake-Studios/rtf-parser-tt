# rtf-parser-tt

[![Crates.io](https://img.shields.io/crates/v/rtf-parser-tt.svg)](https://crates.io/crates/rtf-parser-tt)
[![Documentation](https://docs.rs/rtf-parser-tt/badge.svg)](https://docs.rs/rtf-parser-tt)
[![License](https://img.shields.io/crates/l/rtf-parser-tt.svg)](LICENSE.md)

RTF parser with special character support. Fork of [rtf-parser](https://crates.io/crates/rtf-parser) by [@d0rianb](https://github.com/d0rianb).

## Why This Fork?

The upstream `rtf-parser` crate silently drops special character control words like `\emdash`, `\endash`, and smart quotes. This causes **data loss** when parsing RTF from applications like Scrivener, Microsoft Word, and others.

This fork adds support for these characters.

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
    let text = doc.to_text();
    assert!(text.contains("—")); // Em-dash preserved!
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

## Upstream Status

A PR has been submitted to the upstream `rtf-parser` repository. Once merged, this fork may be deprecated in favor of the upstream version.

## License

MIT License - same as upstream.

## Credits

- Original crate: [rtf-parser](https://github.com/d0rianb/rtf-parser) by [@d0rianb](https://github.com/d0rianb)
- Fork maintained by [TwelveTake Studios](https://twelvetake.com)
