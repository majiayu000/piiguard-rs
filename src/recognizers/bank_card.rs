use super::credit_card::CARD_PATTERN;
use super::{ID_FIELD_PENALTY, ID_FIELD_WORDS};
use super::{PatternRecognizer, strip_separators};
use crate::checksum::luhn_valid;

const CONTEXT: &[&str] = &[
    "银行卡",
    "卡号",
    "账号",
    "帐号",
    "账户",
    "开户",
    "转账",
    "汇款",
    "收款",
    "打款",
    "借记卡",
    "储蓄卡",
    "bank",
    "card",
    "account",
];

/// 中国大陆银行卡：16–19 位 + Luhn。单靠 Luhn 时随机数字串有 1/10 概率通过，
/// 所以无上下文时基础分低于默认阈值 0.5，只有出现"卡号/银行卡"等词才会报出。
fn validate(s: &str) -> Option<f32> {
    let d = strip_separators(s);
    if !(16..=19).contains(&d.len()) || !luhn_valid(&d) {
        return None;
    }
    Some(if d.starts_with("62") { 0.45 } else { 0.35 })
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("CN_BANK_CARD", CARD_PATTERN, 0.35)?
        .validator(validate)
        .context(CONTEXT, 0.4)
        .negative_context(ID_FIELD_WORDS, ID_FIELD_PENALTY))
}
