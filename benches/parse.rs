use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rtf_parser_tt::{Lexer, RtfDocument};

fn formatted_document(paragraphs: usize) -> String {
    let mut s = String::from(
        "{\\rtf1\\ansi\\ansicpg1252\\deff0{\\fonttbl{\\f0\\froman\\fcharset0 Times New Roman;}{\\f1\\fswiss\\fcharset0 Arial;}}{\\colortbl;\\red255\\green0\\blue0;\\red0\\green0\\blue255;}\n",
    );
    for i in 0..paragraphs {
        s.push_str(&format!(
            "\\pard\\sa120\\f0\\fs24 Paragraph {i}: the {{\\b quick}} brown {{\\i fox}} jumps over the {{\\cf1 lazy}} dog\\emdash again and again. He said \\ldblquote caf\\'e9\\rdblquote  and \\u8212? then \\u21834? ok.\\par\n"
        ));
    }
    s.push('}');
    s
}

fn image_document(bytes: u32) -> String {
    let mut s = String::from("{\\rtf1\\ansi Before{\\pict\\pngblip ");
    for i in 0..bytes {
        s.push_str(&format!("{:02x}", i % 256));
        if i % 64 == 63 {
            s.push('\n');
        }
    }
    s.push_str("}After}");
    s
}

fn bench(c: &mut Criterion) {
    let inputs = [
        ("sample-500kB", include_str!("../resources/tests/file-sample_500kB.rtf").to_string()),
        ("formatted-4MB", formatted_document(20_000)),
        ("image-2MB", image_document(1_000_000)),
    ];
    let mut group = c.benchmark_group("parse");
    group.sample_size(20);
    for (name, rtf) in &inputs {
        group.throughput(Throughput::Bytes(rtf.len() as u64));
        group.bench_function(format!("{name}/lex"), |b| b.iter(|| Lexer::scan(black_box(rtf)).map(|t| t.len()).unwrap_or(0)));
        group.bench_function(format!("{name}/document"), |b| {
            b.iter(|| RtfDocument::try_from(black_box(rtf.as_str())).map(|d| d.body.len()).unwrap_or(0))
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
