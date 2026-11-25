use super::PatternRecognizer;
use super::cn_id::province_ok;
use crate::checksum::gb32100_valid;

/// 登记管理部门码 + 机构类别码 + 6 位行政区划码 + 9 位组织机构代码 + 1 位校验码。
const PATTERN: &str = r"[0-9A-HJ-NPQRTUWXY]{2}[0-9]{6}[0-9A-HJ-NPQRTUWXY]{10}";

const CONTEXT: &[&str] = &["信用代码", "统一社会", "税号", "营业执照", "uscc"];

fn validate(s: &str) -> Option<f32> {
    (province_ok(&s[2..4]) && gb32100_valid(s)).then_some(0.85)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("CN_USCC", PATTERN, 0.85)?
        .validator(validate)
        .context(CONTEXT, 0.1))
}
