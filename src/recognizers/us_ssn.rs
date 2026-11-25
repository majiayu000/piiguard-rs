use super::PatternRecognizer;

const PATTERN: &str = r"[0-9]{3}-[0-9]{2}-[0-9]{4}|[0-9]{3} [0-9]{2} [0-9]{4}|[0-9]{9}";

const CONTEXT: &[&str] = &["ssn", "social security", "社会安全"];

/// 公开用于广告/示例、已被 SSA 作废的号码。
const KNOWN_INVALID: &[&str] = &["078051120", "219099999", "123456789"];

/// SSN 没有校验位，只能排除无效号段：区号 000/666/9xx、组号 00、序号 0000。
/// 9 位连续数字太常见，无上下文时分数低于默认阈值。
fn validate(s: &str) -> Option<f32> {
    let d: String = s.chars().filter(char::is_ascii_digit).collect();
    let (area, group, serial) = (&d[..3], &d[3..5], &d[5..]);
    let invalid = area == "000"
        || area == "666"
        || area.starts_with('9')
        || group == "00"
        || serial == "0000"
        || KNOWN_INVALID.contains(&d.as_str());
    if invalid {
        return None;
    }
    Some(if d.len() == s.len() { 0.3 } else { 0.6 })
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("US_SSN", PATTERN, 0.6)?
        .validator(validate)
        .context(CONTEXT, 0.35))
}
