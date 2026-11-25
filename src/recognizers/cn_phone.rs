use super::{PatternRecognizer, strip_separators};

const PATTERN: &str = r"(?:\+86[ \-]?|0086[ \-]?)?1[3-9][0-9](?:[ \-]?[0-9]{4}){2}";

const CONTEXT: &[&str] = &[
    "手机", "电话", "联系", "号码", "致电", "tel", "phone", "mobile",
];

/// 已分配的移动/联通/电信/广电号段前三位。
fn segment_ok(p: &str) -> bool {
    matches!(
        p.parse::<u32>(),
        Ok(
            130..=139
            | 145..=149
            | 150..=153
            | 155..=159
            | 162
            | 165..=167
            | 170..=178
            | 180..=189
            | 190..=193
            | 195..=199,
        )
    )
}

/// 11 位手机号没有校验位，只校验号段；无上下文时分数刚过默认阈值。
fn validate(s: &str) -> Option<f32> {
    let d = strip_separators(s);
    let d = d
        .strip_prefix("+86")
        .or_else(|| d.strip_prefix("0086"))
        .unwrap_or(&d);
    (d.len() == 11 && segment_ok(&d[..3])).then_some(0.55)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("CN_PHONE", PATTERN, 0.55)?
        .validator(validate)
        .context(CONTEXT, 0.35))
}
