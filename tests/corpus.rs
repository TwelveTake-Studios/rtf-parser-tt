use std::fs;
use std::panic;
use std::path::{Path, PathBuf};

use rtf_parser_tt::RtfDocument;

fn corpus_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources").join("corpus");
    let mut files = Vec::new();
    for producer in ["word", "libreoffice"] {
        for entry in fs::read_dir(root.join(producer)).expect("corpus folder") {
            let path = entry.expect("corpus entry").path();
            if path.extension().is_some_and(|ext| ext == "rtf") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[test]
fn corpus_is_present() {
    assert_eq!(corpus_files().len(), 17);
}

#[test]
#[ignore = "phase 1: no panics (libreoffice/international.rtf panics in 0.5.0)"]
fn corpus_parses_without_panic() {
    panic::set_hook(Box::new(|_| {}));
    let mut panicked = Vec::new();
    for path in corpus_files() {
        let text = String::from_utf8_lossy(&fs::read(&path).expect("corpus file")).into_owned();
        if panic::catch_unwind(move || RtfDocument::try_from(text.as_str()).map(|d| d.get_text()).map_err(|e| e.to_string())).is_err() {
            panicked.push(path.display().to_string());
        }
    }
    let _ = panic::take_hook();
    assert!(panicked.is_empty(), "panicked on {panicked:?}");
}
