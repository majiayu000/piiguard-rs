//! 识别器：`Recognizer` trait、通用的 `PatternRecognizer`，以及内置识别器注册表。

mod bank_card;
mod cn_id;
mod cn_phone;
mod cn_uscc;
mod context;
mod credit_card;
mod email;
mod iban;
mod ip;
mod pattern;
mod url_token;
mod us_phone;
mod us_ssn;

pub use pattern::PatternRecognizer;

/// 一次命中。`start`/`end` 为 UTF-8 字节偏移，`char_start`/`char_end` 为字符（Unicode 标量）偏移。
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Finding {
    pub entity: String,
    pub start: usize,
    pub end: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub score: f32,
}

impl Finding {
    /// 字符偏移由 `Analyzer` 统一回填，识别器只需给字节偏移。
    pub fn new(entity: impl Into<String>, start: usize, end: usize, score: f32) -> Self {
        Self {
            entity: entity.into(),
            start,
            end,
            char_start: 0,
            char_end: 0,
            score,
        }
    }
}

/// 识别器扩展点（自定义识别器、未来的 NER 都实现它）。
pub trait Recognizer: Send + Sync {
    fn entity(&self) -> &str;

    fn find(&self, text: &str, out: &mut Vec<Finding>);
}

/// 内置识别器的实体名。
pub const BUILTIN_ENTITIES: &[&str] = &[
    "EMAIL",
    "IP_ADDRESS",
    "CREDIT_CARD",
    "IBAN",
    "URL_TOKEN",
    "US_SSN",
    "US_PHONE",
    "CN_ID_CARD",
    "CN_PHONE",
    "CN_USCC",
    "CN_BANK_CARD",
];

/// 按实体名构造内置识别器；未知实体名返回 `Ok(None)`。
pub fn builtin(entity: &str) -> Result<Option<PatternRecognizer>, regex::Error> {
    let r = match entity {
        "EMAIL" => email::recognizer()?,
        "IP_ADDRESS" => ip::recognizer()?,
        "CREDIT_CARD" => credit_card::recognizer()?,
        "IBAN" => iban::recognizer()?,
        "URL_TOKEN" => url_token::recognizer()?,
        "US_SSN" => us_ssn::recognizer()?,
        "US_PHONE" => us_phone::recognizer()?,
        "CN_ID_CARD" => cn_id::recognizer()?,
        "CN_PHONE" => cn_phone::recognizer()?,
        "CN_USCC" => cn_uscc::recognizer()?,
        "CN_BANK_CARD" => bank_card::recognizer()?,
        _ => return Ok(None),
    };
    Ok(Some(r))
}

/// 订单号、流水号等字段名：数字串紧跟在它们后面时多半不是 PII（负向上下文）。
pub(crate) const ID_FIELD_WORDS: &[&str] = &[
    "order",
    "order_id",
    "orderid",
    "order no",
    "tracking",
    "trace_id",
    "txn",
    "invoice",
    "订单",
    "订单号",
    "单号",
    "流水号",
    "运单号",
];

/// 负向上下文命中时的扣分：足以把任何内置识别器的基础分压到默认阈值以下。
pub(crate) const ID_FIELD_PENALTY: f32 = 0.5;

/// 去掉空格和连字符，供卡号类校验使用。
pub(crate) fn strip_separators(s: &str) -> String {
    s.chars().filter(|c| *c != ' ' && *c != '-').collect()
}
