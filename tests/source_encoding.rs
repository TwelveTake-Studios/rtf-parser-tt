use std::fs;
use std::path::Path;

#[test]
fn test_sources_are_plain_ascii() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found = Vec::new();
    for dir in ["tests", "examples", "benches"] {
        let Ok(entries) = fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries {
            let path = entry.expect("directory entry").path();
            if !path.extension().is_some_and(|ext| ext == "rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("source file");
            for (number, line) in text.lines().enumerate() {
                let unexpected = line.chars().find(|c| !c.is_ascii() || (c.is_ascii_control() && *c != '\t' && *c != '\r'));
                if let Some(c) = unexpected {
                    found.push(format!("{}:{}: U+{:04X}", path.display(), number + 1, c as u32));
                }
            }
        }
    }
    assert!(found.is_empty(), "write these characters as \\u{{...}} escapes: {found:?}");
}
