mod support;

use piiguard::{Analyzer, Error, Finding, PatternRecognizer, Recognizer};

#[test]
fn threshold_controls_low_score_hits() {
    let card = support::unionpay();
    let strict = Analyzer::builder()
        .with_builtin(&["CN_BANK_CARD"])
        .build()
        .unwrap();
    assert!(strict.analyze(&card).is_empty());
    let loose = Analyzer::builder()
        .with_builtin(&["CN_BANK_CARD"])
        .threshold(0.4)
        .build()
        .unwrap();
    assert_eq!(loose.analyze(&card).len(), 1);
}

#[test]
fn overlap_keeps_higher_score() {
    let a = Analyzer::builder()
        .with_recognizer(PatternRecognizer::new("LOW", "abc def", 0.6).unwrap())
        .with_recognizer(PatternRecognizer::new("HIGH", "def", 0.9).unwrap())
        .build()
        .unwrap();
    let f = a.analyze("abc def");
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].entity, "HIGH");
}

#[test]
fn results_sorted_by_position() {
    let a = Analyzer::builder().with_all_builtin().build().unwrap();
    let text = format!("x@example.com 和 {} 和 y@example.com", support::id_card());
    let f = a.analyze(&text);
    let entities: Vec<&str> = f.iter().map(|f| f.entity.as_str()).collect();
    assert_eq!(entities, ["EMAIL", "CN_ID_CARD", "EMAIL"]);
    assert!(f.windows(2).all(|w| w[0].end <= w[1].start));
}

#[test]
fn byte_and_char_offsets() {
    let a = Analyzer::builder()
        .with_builtin(&["EMAIL"])
        .build()
        .unwrap();
    let f = a.analyze("你好 a@example.com");
    assert_eq!((f[0].start, f[0].end), (7, 20));
    assert_eq!((f[0].char_start, f[0].char_end), (3, 16));
}

#[test]
fn allow_list_suppresses_exact_value() {
    let a = Analyzer::builder()
        .with_builtin(&["EMAIL"])
        .allow(&["noreply@example.com"])
        .build()
        .unwrap();
    let f = a.analyze("from noreply@example.com to bob@example.com");
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].start, 28);
}

#[test]
fn deny_list_always_reports() {
    let a = Analyzer::builder()
        .deny("CODENAME", &["Project Phoenix", "凤凰计划"])
        .build()
        .unwrap();
    let f = a.analyze("关于凤凰计划和 Project Phoenix (v1.2)");
    let names: Vec<(&str, f32)> = f.iter().map(|f| (f.entity.as_str(), f.score)).collect();
    assert_eq!(names, [("CODENAME", 1.0), ("CODENAME", 1.0)]);
}

#[test]
fn unknown_builtin_is_error() {
    let r = Analyzer::builder().with_builtin(&["NOPE"]).build();
    assert!(matches!(r, Err(Error::UnknownEntity(e)) if e == "NOPE"));
}

struct EveryX;

impl Recognizer for EveryX {
    fn entity(&self) -> &str {
        "X"
    }
    fn find(&self, text: &str, out: &mut Vec<Finding>) {
        out.extend(
            text.match_indices('x')
                .map(|(i, _)| Finding::new("X", i, i + 1, 1.0)),
        );
    }
}

#[test]
fn custom_recognizer_is_called() {
    let a = Analyzer::builder().with_recognizer(EveryX).build().unwrap();
    assert_eq!(a.analyze("axbx").len(), 2);
}
