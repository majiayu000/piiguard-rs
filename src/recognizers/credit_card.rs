use super::{ID_FIELD_PENALTY, ID_FIELD_WORDS};
use super::{PatternRecognizer, strip_separators};
use crate::checksum::luhn_valid;

/// 卡号形态：连续 13–19 位，或按 4 位分组（空格/连字符），或 Amex 的 4-6-5 分组。
pub(super) const CARD_PATTERN: &str = concat!(
    r"[0-9]{4}(?:[ \-][0-9]{4}){2,3}(?:[ \-][0-9]{1,4})?",
    r"|[0-9]{4}[ \-][0-9]{6}[ \-][0-9]{5}",
    r"|[0-9]{13,19}",
);

const CONTEXT: &[&str] = &[
    "card",
    "credit",
    "visa",
    "mastercard",
    "amex",
    "cvv",
    "信用卡",
    "卡号",
];

fn prefix_in(d: &str, len: usize, lo: u32, hi: u32) -> bool {
    d.get(..len)
        .and_then(|p| p.parse::<u32>().ok())
        .is_some_and(|p| (lo..=hi).contains(&p))
}

/// 已知国际发卡组织的号段与长度。
fn issuer_ok(d: &str) -> bool {
    let n = d.len();
    let visa = d.starts_with('4') && matches!(n, 13 | 16 | 19);
    let mastercard = n == 16 && (prefix_in(d, 2, 51, 55) || prefix_in(d, 4, 2221, 2720));
    let amex = n == 15 && (d.starts_with("34") || d.starts_with("37"));
    let discover = (16..=19).contains(&n)
        && (d.starts_with("6011") || d.starts_with("65") || prefix_in(d, 3, 644, 649));
    let jcb = (16..=19).contains(&n) && prefix_in(d, 4, 3528, 3589);
    visa || mastercard || amex || discover || jcb
}

fn validate(s: &str) -> Option<f32> {
    let d = strip_separators(s);
    (issuer_ok(&d) && luhn_valid(&d)).then_some(0.7)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("CREDIT_CARD", CARD_PATTERN, 0.7)?
        .validator(validate)
        .context(CONTEXT, 0.25)
        .negative_context(ID_FIELD_WORDS, ID_FIELD_PENALTY))
}
