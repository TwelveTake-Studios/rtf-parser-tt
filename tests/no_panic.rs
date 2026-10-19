use std::panic;

use rtf_parser_tt::RtfDocument;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len() as u64) as usize]
    }
}

const WORDS: &[&str] = &[
    "rtf", "ansi", "ansicpg", "mac", "pc", "pca", "u", "uc", "fonttbl", "colortbl", "stylesheet", "f", "fs", "fcharset", "cf", "b", "i", "ul",
    "ulnone", "par", "pard", "plain", "line", "tab", "page", "sect", "cell", "row", "trowd", "cellx", "li", "ri", "fi", "sb", "sa", "sl", "slmult",
    "red", "green", "blue", "emdash", "endash", "bullet", "lquote", "rquote", "ldblquote", "rdblquote", "pict", "bin", "info", "title", "v",
    "field", "fldinst", "fldrslt", "s", "xyz",
];

const TEXT: &[&str] = &["a", "word", " ", "  ", ";", ",", "0", "-", "Р", "х", "é", "中", "😀", "\u{85}", "\u{a0}", "\n", "\r\n", "\t"];

const SYMBOLS: &[&str] = &["\\~", "\\-", "\\_", "\\|", "\\:", "\\*", "\\\\", "\\{", "\\}", "\\\n", "\\", "\\'", "\\'9", "\\'93", "\\'zz", "\\'Р"];

fn piece(rng: &mut Rng, out: &mut String) {
    match rng.below(7) {
        0 => out.push('{'),
        1 => out.push('}'),
        2 => {
            out.push('\\');
            out.push_str(rng.pick(WORDS));
            match rng.below(4) {
                0 => {}
                1 => out.push_str(&(rng.below(70000) as i64 - 35000).to_string()),
                2 => out.push_str(&rng.below(10).to_string()),
                _ => out.push_str("99999999999"),
            }
            if rng.below(2) == 0 {
                out.push(' ');
            }
        }
        3 => out.push_str(rng.pick(SYMBOLS)),
        4 => {
            out.push_str("\\u");
            out.push_str(&(rng.below(140000) as i64 - 70000).to_string());
            out.push_str(rng.pick(&["", " ", "?", "\\'3f", "\\'93\\'94"]));
        }
        _ => out.push_str(rng.pick(TEXT)),
    }
}

fn document(rng: &mut Rng) -> String {
    let mut out = String::from(if rng.below(5) == 0 { "" } else { "{\\rtf1" });
    for _ in 0..rng.below(40) {
        piece(rng, &mut out);
    }
    if rng.below(5) != 0 {
        out.push('}');
    }
    out
}

#[test]
fn random_inputs_never_panic() {
    panic::set_hook(Box::new(|_| {}));
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    let mut failures: Vec<String> = Vec::new();
    for _ in 0..20_000 {
        let doc = document(&mut rng);
        let owned = doc.clone();
        if panic::catch_unwind(move || RtfDocument::try_from(owned.as_str()).map(|d| d.get_text())).is_err() {
            failures.push(doc);
        }
    }
    let _ = panic::take_hook();
    failures.sort_by_key(|d| d.len());
    assert!(failures.is_empty(), "{} of 20000 inputs panicked; shortest: {:?}", failures.len(), &failures[..failures.len().min(5)]);
}
