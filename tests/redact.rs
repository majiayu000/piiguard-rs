use piiguard::{Action, Analyzer, Vault, redact, restore};

fn analyzer() -> Analyzer {
    Analyzer::builder()
        .with_builtin(&["EMAIL", "CN_PHONE"])
        .build()
        .unwrap()
}

const TEXT: &str = "张三 a@example.com，手机13812345678，再发 a@example.com";

#[test]
fn mask_keeps_char_count() {
    let f = analyzer().analyze(TEXT);
    let out = redact(TEXT, &f, Action::Mask { ch: '*' });
    assert_eq!(
        out,
        "张三 *************，手机***********，再发 *************"
    );
    assert_eq!(out.chars().count(), TEXT.chars().count());
}

#[test]
fn replace_numbers_same_value_identically() {
    let f = analyzer().analyze(TEXT);
    let out = redact(TEXT, &f, Action::Replace);
    assert_eq!(out, "张三 <EMAIL_1>，手机<CN_PHONE_1>，再发 <EMAIL_1>");
}

#[test]
fn pseudonymize_is_stable_across_calls_and_restorable() {
    let a = analyzer();
    let mut vault = Vault::new();
    let first = redact(TEXT, &a.analyze(TEXT), Action::Pseudonymize(&mut vault));
    let second_text = "回复 b@example.com 和 a@example.com";
    let second = redact(
        second_text,
        &a.analyze(second_text),
        Action::Pseudonymize(&mut vault),
    );
    assert_eq!(second, "回复 <EMAIL_2> 和 <EMAIL_1>");
    assert_eq!(vault.len(), 3);
    assert_eq!(restore(&first, &vault), TEXT);
    assert_eq!(restore(&second, &vault), second_text);
}

#[test]
fn literal_placeholder_in_source_survives_roundtrip() {
    let text = "模板里有 <EMAIL_1>，真实邮箱 a@example.com";
    let mut vault = Vault::new();
    let out = redact(
        text,
        &analyzer().analyze(text),
        Action::Pseudonymize(&mut vault),
    );
    assert_eq!(out, "模板里有 <EMAIL_1>，真实邮箱 <EMAIL_2>");
    assert_eq!(restore(&out, &vault), text);
}

#[test]
fn restore_leaves_unknown_placeholders() {
    let vault = Vault::new();
    assert_eq!(
        restore("<EMAIL_9> a < b > c", &vault),
        "<EMAIL_9> a < b > c"
    );
}

#[test]
fn overlapping_or_misaligned_findings_are_skipped() {
    use piiguard::Finding;
    let text = "你好世界";
    let f = [
        Finding::new("X", 1, 3, 1.0),
        Finding::new("X", 0, 3, 1.0),
        Finding::new("X", 0, 3, 1.0),
    ];
    assert_eq!(redact(text, &f, Action::Replace), "<X_1>好世界");
}
