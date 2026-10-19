use std::fs;
use std::panic;
use std::path::Path;

use rtf_parser_tt::RtfDocument;

fn normalize(text: &str) -> Vec<char> {
    text.trim_start_matches('\u{feff}')
        .replace("\r\n", "\n")
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .collect()
}

fn similarity(a: &[char], b: &[char]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let mut prev = vec![0usize; b.len() + 1];
    let mut cur = vec![0usize; b.len() + 1];
    for &x in a {
        for (j, &y) in b.iter().enumerate() {
            cur[j + 1] = if x == y { prev[j] + 1 } else { cur[j].max(prev[j + 1]) };
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    2.0 * prev[b.len()] as f64 / (a.len() + b.len()) as f64
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources").join("corpus");
    let mut total = 0.0;
    let mut count = 0;
    println!("{:<14} {:<20} {:>8}  status", "producer", "document", "match");
    for producer in ["word", "libreoffice"] {
        let mut entries: Vec<_> = fs::read_dir(root.join(producer)).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for rtf_path in entries.into_iter().filter(|p| p.extension().is_some_and(|e| e == "rtf")) {
            let reference = fs::read_to_string(rtf_path.with_extension("txt")).unwrap_or_default();
            let source = String::from_utf8_lossy(&fs::read(&rtf_path).unwrap()).into_owned();
            let parsed = panic::catch_unwind(move || RtfDocument::try_from(source.as_str()).map(|d| d.get_text()).map_err(|e| e.to_string()));
            let (score, status) = match parsed {
                Ok(Ok(text)) => (similarity(&normalize(&text), &normalize(&reference)), "ok".to_string()),
                Ok(Err(err)) => (0.0, format!("error: {err}")),
                Err(_) => (0.0, "panic".to_string()),
            };
            total += score;
            count += 1;
            let name = rtf_path.file_stem().unwrap().to_string_lossy().into_owned();
            println!("{:<14} {:<20} {:>7.1}%  {}", producer, name, score * 100.0, status);
        }
    }
    println!("{:<14} {:<20} {:>7.1}%", "all", "mean", total / count as f64 * 100.0);
}
