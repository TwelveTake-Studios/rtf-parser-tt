# Changelog

All notable changes to this project are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
