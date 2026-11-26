use super::PatternRecognizer;
use super::cn_id::province_ok;
use super::{ID_FIELD_PENALTY, ID_FIELD_WORDS};
use crate::checksum::gb32100_valid;

/// 登记管理部门码 + 机构类别码 + 6 位行政区划码 + 9 位组织机构代码 + 1 位校验码。
const PATTERN: &str = r"[0-9A-HJ-NPQRTUWXY]{2}[0-9]{6}[0-9A-HJ-NPQRTUWXY]{10}";

const CONTEXT: &[&str] = &["信用代码", "统一社会", "税号", "营业执照", "uscc"];

/// GB 32100 表 1/表 2：登记管理部门码与机构类别码的合法组合。
fn dept_category_ok(dept: u8, category: u8) -> bool {
    let allowed: &[u8] = match dept {
        b'1' | b'5' | b'N' => b"1239",
        b'2' | b'4' | b'8' | b'A' => b"19",
        b'3' => b"123459",
        b'6' | b'7' => b"129",
        b'9' => b"123",
        b'Y' => b"1",
        _ => b"",
    };
    allowed.contains(&category)
}

fn validate(s: &str) -> Option<f32> {
    let b = s.as_bytes();
    let ok = dept_category_ok(b[0], b[1]) && province_ok(&s[2..4]) && gb32100_valid(s);
    ok.then_some(0.85)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("CN_USCC", PATTERN, 0.85)?
        .validator(validate)
        .context(CONTEXT, 0.1)
        .negative_context(ID_FIELD_WORDS, ID_FIELD_PENALTY))
}
