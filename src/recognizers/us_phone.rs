use super::PatternRecognizer;
use super::{ID_FIELD_PENALTY, ID_FIELD_WORDS};

const PATTERN: &str = r"(?:\+1[ .\-]?)?(?:\([0-9]{3}\) ?|[0-9]{3}[ .\-]?)[0-9]{3}[ .\-]?[0-9]{4}";

const CONTEXT: &[&str] = &["phone", "tel", "call", "mobile", "cell", "fax", "电话"];

/// NANP 规则：区号与交换码首位为 2–9，且都不是 N11（如 911）。
fn nxx_ok(x: &[u8]) -> bool {
    (b'2'..=b'9').contains(&x[0]) && !(x[1] == b'1' && x[2] == b'1')
}

/// 有分隔符/括号的格式更像电话，纯 10 位数字分数低于默认阈值。
fn validate(s: &str) -> Option<f32> {
    let mut d: Vec<u8> = s.bytes().filter(u8::is_ascii_digit).collect();
    if d.len() == 11 {
        d.remove(0);
    }
    if d.len() != 10 || !nxx_ok(&d[..3]) || !nxx_ok(&d[3..6]) {
        return None;
    }
    let formatted = s.bytes().any(|b| !b.is_ascii_digit());
    Some(if formatted { 0.5 } else { 0.3 })
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("US_PHONE", PATTERN, 0.5)?
        .validator(validate)
        .context(CONTEXT, 0.3)
        .negative_context(ID_FIELD_WORDS, ID_FIELD_PENALTY))
}
