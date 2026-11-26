use piiguard::{Action, Analyzer, StreamRestorer, Vault, redact, restore};
use proptest::prelude::*;

fn vault_with(text: &str) -> (String, Vault) {
    let a = Analyzer::builder()
        .with_builtin(&["EMAIL", "CN_PHONE"])
        .build()
        .unwrap();
    let mut vault = Vault::new();
    let redacted = redact(text, &a.analyze(text), Action::Pseudonymize(&mut vault));
    (redacted, vault)
}

fn stream(vault: &Vault, chunks: &[&str]) -> String {
    let mut r = StreamRestorer::new(vault);
    let mut out: String = chunks.iter().map(|c| r.push(c)).collect();
    out.push_str(&r.finish());
    out
}

#[test]
fn placeholder_split_across_chunks_is_restored() {
    let (_, vault) = vault_with("a@example.com 13812345678");
    assert_eq!(
        stream(&vault, &["回复 <EMA", "IL_1> 和 <CN_", "PHONE_1", ">。"]),
        "回复 a@example.com 和 13812345678。"
    );
}

#[test]
fn text_without_placeholders_is_emitted_immediately() {
    let (_, vault) = vault_with("a@example.com");
    let mut r = StreamRestorer::new(&vault);
    assert_eq!(r.push("x < y and "), "x < y and ");
    assert_eq!(r.push("<EM"), "");
    assert_eq!(r.finish(), "<EM");
}

#[test]
fn long_unclosed_angle_bracket_is_not_held_forever() {
    let (_, vault) = vault_with("a@example.com");
    let mut r = StreamRestorer::new(&vault);
    let long = format!("<{}", "x".repeat(64));
    assert_eq!(r.push(&long), long);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn any_chunking_equals_whole_restore(
        filler in prop::collection::vec("[ -~，。]{0,6}", 1..5),
        cuts in prop::collection::vec(0usize..200, 0..12),
    ) {
        let original = format!("{} a@example.com {} 13812345678 {}", filler[0], filler.join("<"), "b@example.org");
        let (redacted, vault) = vault_with(&original);
        let mut points: Vec<usize> = cuts.into_iter()
            .map(|c| c % (redacted.len() + 1))
            .filter(|&c| redacted.is_char_boundary(c))
            .collect();
        points.sort_unstable();
        points.dedup();
        let mut chunks = Vec::new();
        let mut last = 0;
        for p in points.into_iter().chain([redacted.len()]) {
            chunks.push(&redacted[last..p]);
            last = p;
        }
        prop_assert_eq!(stream(&vault, &chunks), restore(&redacted, &vault));
        prop_assert_eq!(restore(&redacted, &vault), original);
    }
}
