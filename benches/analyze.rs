use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use piiguard::{Action, Analyzer, Vault, redact};

#[path = "../tests/support/mod.rs"]
mod support;

/// 合成语料：大部分是无 PII 的日志行，少量混入合成号码。
fn corpus() -> String {
    let mut s = String::new();
    for i in 0..2000 {
        s.push_str(&format!(
            "2026-11-26T12:{:02}:{:02}Z INFO request_id={i:08} path=/v1/chat latency_ms={}\n",
            i % 60,
            (i * 7) % 60,
            i % 900
        ));
        if i % 50 == 0 {
            s.push_str(&format!(
                "用户 bob{i}@example.com 身份证 {} 卡号 {}\n",
                support::id_card(),
                support::visa()
            ));
        }
    }
    s
}

fn bench(c: &mut Criterion) {
    let analyzer = Analyzer::builder().with_all_builtin().build().unwrap();
    let text = corpus();
    let mut g = c.benchmark_group("analyze");
    g.throughput(Throughput::Bytes(text.len() as u64));
    g.bench_function("all_builtin", |b| b.iter(|| analyzer.analyze(&text)));
    g.bench_function("analyze_and_pseudonymize", |b| {
        b.iter(|| {
            let mut vault = Vault::new();
            let f = analyzer.analyze(&text);
            redact(&text, &f, Action::Pseudonymize(&mut vault))
        })
    });
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
