# Changelog

All notable changes to this project are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-10-21

### Added

- `RtfDocument::from_bytes` parses an RTF document from bytes. UTF-8 input is read as is; any
  other input is decoded with the code page the document declares. `RtfDocument::from_filepath`
  and `TryFrom<&mut File>` read files the same way, so a file no longer needs to be valid UTF-8.
- `RtfDocument::get_text_with` returns the plain text according to `TextOptions`: hidden text
  (off by default), footnotes with their numbers (on by default), and headers and footers (on by
  default). `RtfDocument::get_text` uses the defaults.
- `StyleBlock::part` identifies the part of the document a block belongs to: `Part::Body`,
  `Part::NoteReference(n)` for a footnote number in the body text, `Part::Note(n)`,
  `Part::Header` or `Part::Footer`.
- `RtfDocument::info` holds the document properties from the `\info` group: title, subject,
  author, manager, company, operator, category, keywords and comment.
- `\'xx` escapes are decoded through the code page that `\ansicpg`, `\mac`, `\pc` or `\pca`
  declares, or that the current font's `\fcharset` selects: Windows code pages 874 and 1250 to
  1258, DOS code pages 437, 850 and 866, Mac Roman, UTF-8, and the Symbol font.
- The `encoding_rs` feature, enabled by default, decodes the Japanese (932), Simplified Chinese
  (936), Korean (949) and Traditional Chinese (950) code pages.
- `RtfError` is the error type of `parse_rtf` and of the `RtfDocument` constructors. It implements
  `std::error::Error` and is `Send` and `Sync`, and `LexerError` and `ParserError` convert into it.
- `Painter::hidden` marks hidden text (`\v`).
- `Style` exposes the name, character formatting and paragraph properties of a stylesheet entry.
- Footnote numbers (`\chftn`) appear in the text.

### Changed

- `RtfDocument::try_from`, `RtfDocument::from_filepath` and `parse_rtf` return `RtfError` on
  failure. `parse_rtf` returns a `Result`; it previously returned the document directly.
- Public enums and structs are marked `#[non_exhaustive]`. A `match` on an enum from this crate
  needs a wildcard arm, and a struct is created from its default value and then modified.
  `TextOptions` sets each option with a method, for example
  `TextOptions::default().hidden_text(true)`.
- `Lexer::scan` reports `\v`, `\sect`, `\page`, `\cell`, `\row` and 11 other control words with
  their own `ControlWord` variants. 0.5.x reported them as `ControlWord::Unknown`.
- Paragraph, section and page breaks (`\par`, `\sect`, `\page`) produce a line break, as `\line`
  does. Table cells are separated by a tab, and each table row ends with a line break. In 0.5.x,
  paragraphs and table cells ran together.
- A `StyleBlock` continues across paragraph breaks while the formatting stays the same, so its
  text can contain line breaks between other text. Code that treats a block containing a line
  break as a paragraph break alone must split the text at each line break.
- Characters that follow `\uN` are skipped according to `\ucN` (one character by default), as the
  RTF specification requires. 0.5.x kept them in the text: `\u8212 ?` produced an em dash
  followed by a question mark. A document that writes no replacement character without first
  setting `\uc0` loses the character after each `\uN`.
- Hidden text (`\v`) is omitted from `get_text`. `get_text_with` includes it when
  `TextOptions::hidden_text` is set.
- Footnotes, headers and footers follow the body text, in the order Word uses when it saves a
  document as plain text. In 0.5.x their text appeared where it occurs in the file, inside the
  body text.
- Picture data (`\pict`) and document properties (`\info`) no longer appear in the text.
- Only the space that ends a control word is consumed. A further space, or a tab after a control
  word, is text, and whitespace-only groups such as `{ }` are kept.
- Parsing takes about half the time it took in 0.5.1, or less, on the crate's benchmark
  documents.

### Removed

- The `recursive_tokenize!` and `recursive_tokenize_with_init!` macros.

### Fixed

- `parse_rtf` returns an error for malformed input instead of panicking.
- Text that directly follows a control word is kept when it begins with punctuation or a
  non-ASCII character, for example a comma after `\b0` or Cyrillic text after `\i`. Letters that
  directly follow a control word are part of the control word, as the RTF specification defines,
  so `\iword` remains a single control word.
- `\uN` followed directly by its replacement character, as in `\u8212?`, produces the character.
- The control symbols `\~` (no-break space), `\-` (soft hyphen) and `\_` (non-breaking hyphen)
  produce their characters, and the text after them is kept.
- A control word with a nonzero parameter, such as `\b2`, turns its formatting on.
- Paragraph indents (`\li`, `\ri`, `\fi`) and the line spacing multiple (`\slmult`) are applied
  to `Paragraph`.
- Font names written with hex escapes, or followed by an alternate name (`\*\falt`), are read. In
  0.5.x these names were empty.
- Color table entries are numbered by position, counting empty entries, so `\cfN` selects the
  intended color.
- A document followed by NUL bytes after its closing brace parses.

### Known issues

- Endnotes are numbered in sequence with footnotes and listed among them in file order. Word
  numbers endnotes separately and lists them after the footnotes.
- Text in the Korean Johab code page (1361) is not decoded.

## [0.5.1] - 2026-10-19

### Fixed

- `RtfDocument::try_from` and `RtfDocument::from_filepath` no longer panic on malformed input.
  Unpaired Unicode surrogates decode to U+FFFD, a truncated or invalid `\'xx` escape returns an
  error, and negative or very large `\ucN` values are handled safely.
- Non-ASCII text that directly follows a control word, such as Cyrillic text after `\i`, no
  longer causes a panic.
- Characters outside the Basic Multilingual Plane, such as emoji, decode correctly when a `\ucN`
  control word separates the two halves of the surrogate pair, as in documents saved by
  LibreOffice.
- The README example calls `get_text()`. The previous example did not compile.

### Changed

- The README is now the crate-level documentation, and its example is compiled as a test.
- Output is otherwise unchanged. Compared with 0.5.0 on more than one million generated and
  real-world documents, every document that 0.5.0 parsed produces identical results.

### Known issues

- Text that directly follows a control word without a delimiting space, such as `\iword`, is
  omitted from the output, as in 0.5.0. A fix is planned for the next minor release.
- `parse_rtf` still panics on input that `RtfDocument::try_from` rejects. To receive an error
  instead, call `RtfDocument::try_from`. A fix is planned for the next minor release.

## [0.5.0] - 2025-12-17

### Added

- The special-character control words `\emdash`, `\endash`, `\bullet`, `\lquote`, `\rquote`,
  `\ldblquote`, `\rdblquote`, `\tab` and `\line` produce their characters in the parsed text.
