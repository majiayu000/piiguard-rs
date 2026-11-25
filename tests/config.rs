#![cfg(feature = "serde")]

use piiguard::{Config, Error};

const CFG: &str = r#"{
  "entities": ["EMAIL"],
  "threshold": 0.6,
  "allow": ["noreply@example.com"],
  "deny": [{"entity": "CODENAME", "values": ["凤凰计划"]}],
  "custom": [{"entity": "EMPLOYEE_ID", "pattern": "EMP-[0-9]{6}", "score": 0.5,
              "context": ["工号"], "context_boost": 0.2}]
}"#;

#[test]
fn config_builds_analyzer() {
    let a = Config::from_json(CFG)
        .unwrap()
        .into_builder()
        .unwrap()
        .build()
        .unwrap();
    let text =
        "EMP-000456 无上下文；noreply@example.com 通知 bob@example.com：凤凰计划，工号 EMP-000123";
    let got: Vec<&str> = a
        .analyze(text)
        .iter()
        .map(|f| &text[f.start..f.end])
        .collect();
    assert_eq!(got, ["bob@example.com", "凤凰计划", "EMP-000123"]);
}

#[test]
fn missing_entities_means_all_builtin() {
    let a = Config::from_json("{}")
        .unwrap()
        .into_builder()
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(a.analyze("手机13812345678").len(), 1);
}

#[test]
fn unknown_field_and_bad_pattern_are_errors() {
    assert!(matches!(
        Config::from_json(r#"{"entites": []}"#),
        Err(Error::Config(_))
    ));
    let bad = r#"{"custom": [{"entity": "X", "pattern": "(", "score": 1.0}]}"#;
    let r = Config::from_json(bad).unwrap().into_builder();
    assert!(matches!(r, Err(Error::InvalidPattern(_))));
}
