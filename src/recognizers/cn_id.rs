use super::PatternRecognizer;
use super::{ID_FIELD_PENALTY, ID_FIELD_WORDS};
use crate::checksum::gb11643_valid;

const PATTERN: &str = concat!(
    r"[1-9][0-9]{5}(?:19|20)[0-9]{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12][0-9]|3[01])",
    r"[0-9]{3}[0-9Xx]",
);

const CONTEXT: &[&str] = &["身份证", "证件", "公民身份", "id card", "id no"];

/// 省级行政区划代码前两位（含港澳台居民居住证的 81/82/83）。
pub(super) fn province_ok(code: &str) -> bool {
    matches!(
        code.parse::<u32>(),
        Ok(11..=15 | 21..=23 | 31..=37 | 41..=46 | 50..=54 | 61..=65 | 71 | 81..=83)
    )
}

fn date_ok(year: u32, month: u32, day: u32) -> bool {
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => return false,
    };
    (1..=days).contains(&day)
}

/// 18 位身份证：省份代码 + 出生日期合法 + GB 11643 校验位。
fn validate(s: &str) -> Option<f32> {
    let num = |r: std::ops::Range<usize>| s[r].parse::<u32>().ok();
    let ok = province_ok(&s[..2])
        && date_ok(num(6..10)?, num(10..12)?, num(12..14)?)
        && gb11643_valid(s);
    ok.then_some(0.9)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("CN_ID_CARD", PATTERN, 0.9)?
        .validator(validate)
        .context(CONTEXT, 0.1)
        .negative_context(ID_FIELD_WORDS, ID_FIELD_PENALTY))
}
