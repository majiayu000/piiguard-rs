//! 属性测试：任意文本（混入合成 PII）经 Pseudonymize 后都能被 `restore` 精确还原。
mod support;

use piiguard::{Action, Analyzer, Vault, redact, restore};
use proptest::prelude::*;

fn pii_samples() -> Vec<String> {
    vec![
        "alice@example.com".into(),
        support::id_card(),
        support::visa(),
        support::uscc(),
        support::iban("DE", "370400440532013000"),
        "13812345678".into(),
        "(212) 555-0147".into(),
        "192.168.1.20".into(),
        format!("银行卡 {}", support::unionpay()),
        "<EMAIL_1>".into(),
    ]
}

fn segment() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<String>(),
        "[ -~]{0,20}",
        "[一-龥，。：]{0,8}",
        prop::sample::select(pii_samples()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn restore_inverts_pseudonymize(parts in prop::collection::vec(segment(), 0..8)) {
        let analyzer = Analyzer::builder().with_all_builtin().build().unwrap();
        let text = parts.join(" ");
        let mut vault = Vault::new();
        let redacted = redact(&text, &analyzer.analyze(&text), Action::Pseudonymize(&mut vault));
        prop_assert_eq!(restore(&redacted, &vault), text);
    }

}
