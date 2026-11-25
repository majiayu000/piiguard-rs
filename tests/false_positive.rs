//! 误报基线：无 PII 的语料上，默认阈值下命中数必须为 0。
//! 语料 = 本仓库的 Cargo.lock（版本号、sha256 校验和）+ 按常见网关日志格式生成的行。

use piiguard::Analyzer;

fn log_lines() -> String {
    let mut s = String::new();
    for i in 0u64..3000 {
        let ts_ms = 1_764_000_000_000 + i * 7_919;
        s.push_str(&format!(
            "2026-11-26T12:{:02}:{:02}.{:03}Z INFO ts={ts_ms} req={:016x} order={} \
             latency_ms={} tokens={} model=gpt-4o-2024-08-06 ver=1.{}.{} path=/v1/chat/completions?stream=true\n",
            i % 60,
            (i * 7) % 60,
            i % 1000,
            i.wrapping_mul(0x9E37_79B9_7F4A_7C15),
            20_261_126_000_000 + i,
            i % 900,
            i * 13 % 4096,
            i % 30,
            i % 10,
        ));
    }
    s
}

#[test]
fn no_hits_on_clean_corpus() {
    let a = Analyzer::builder().with_all_builtin().build().unwrap();
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock")).unwrap();
    for (name, text) in [("Cargo.lock", lock), ("gateway log", log_lines())] {
        let hits = a.analyze(&text);
        let sample: Vec<&str> = hits.iter().take(5).map(|f| &text[f.start..f.end]).collect();
        assert!(
            hits.is_empty(),
            "{name}: {} hits, e.g. {sample:?}",
            hits.len()
        );
    }
}
