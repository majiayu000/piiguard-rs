//! 统一 runner：`runner <tool> <out_dir> <corpus.jsonl>...`
//! 每份语料先对前 WARMUP 篇预热，再逐篇计时（只计检测调用本身），
//! 输出 `<out_dir>/<tool>.<corpus>.pred.jsonl`（字符偏移）与 `.timing.json`。

mod tools;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};

const WARMUP: usize = 50;

/// 字节偏移 → 字符偏移。
fn char_offset(text: &str, byte: usize) -> usize {
    text[..byte].chars().count()
}

/// 某些工具会在特定输入上 panic；捕获后记为该篇无输出，并在 timing 里计数。
fn detect_guarded(tool: &tools::Tool, text: &str) -> Option<Vec<(usize, usize, String)>> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tool.detect(text))).ok()
}

fn run_corpus(tool: &tools::Tool, name: &str, corpus: &Path, out_dir: &Path) -> std::io::Result<()> {
    let docs: Vec<Value> = BufReader::new(fs::File::open(corpus)?)
        .lines()
        .map(|l| serde_json::from_str(&l.expect("read line")).expect("valid jsonl"))
        .collect();
    let texts: Vec<&str> = docs.iter().map(|d| d["text"].as_str().expect("text")).collect();
    for t in texts.iter().take(WARMUP) {
        std::hint::black_box(detect_guarded(tool, t));
    }
    let stem = corpus.file_stem().and_then(|s| s.to_str()).unwrap_or("corpus");
    let mut pred = fs::File::create(out_dir.join(format!("{name}.{stem}.pred.jsonl")))?;
    let mut ns = Vec::with_capacity(docs.len());
    let mut panics = 0usize;
    for (doc, text) in docs.iter().zip(&texts) {
        let t0 = Instant::now();
        let spans = std::hint::black_box(detect_guarded(tool, text));
        ns.push(t0.elapsed().as_nanos() as u64);
        let spans = spans.unwrap_or_else(|| {
            panics += 1;
            Vec::new()
        });
        let spans: Vec<Value> = spans
            .into_iter()
            .map(|(s, e, label)| json!({"start": char_offset(text, s), "end": char_offset(text, e), "label": label}))
            .collect();
        writeln!(pred, "{}", json!({"id": doc["id"], "spans": spans}))?;
    }
    let bytes: usize = texts.iter().map(|t| t.len()).sum();
    let timing = json!({"tool": name, "corpus": stem, "docs": docs.len(), "bytes": bytes, "panics": panics, "ns": ns});
    fs::write(out_dir.join(format!("{name}.{stem}.timing.json")), timing.to_string())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: runner <tool> <out_dir> <corpus.jsonl>...  tools: {}", tools::NAMES.join(", "));
        std::process::exit(2);
    }
    std::panic::set_hook(Box::new(|_| {}));
    let t0 = Instant::now();
    let tool = tools::Tool::load(&args[1]).unwrap_or_else(|e| {
        eprintln!("load {}: {e}", args[1]);
        std::process::exit(2)
    });
    let load_ms = t0.elapsed().as_secs_f64() * 1e3;
    let out_dir = Path::new(&args[2]);
    fs::create_dir_all(out_dir)?;
    for c in &args[3..] {
        run_corpus(&tool, &args[1], Path::new(c), out_dir)?;
    }
    fs::write(out_dir.join(format!("{}.load.json", args[1])), json!({"load_ms": load_ms}).to_string())
}
