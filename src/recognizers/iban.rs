use super::PatternRecognizer;
use crate::checksum::iban_valid;

const PATTERN: &str = concat!(
    r"[A-Z]{2}[0-9]{2}(?: [A-Z0-9]{4}){2,7}(?: [A-Z0-9]{1,3})?",
    r"|[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}",
);

/// 常见国家的 IBAN 固定长度（ISO 13616 注册表的子集）；不在表内的国家不报。
const COUNTRY_LEN: &[(&str, usize)] = &[
    ("AD", 24),
    ("AE", 23),
    ("AT", 20),
    ("BE", 16),
    ("BG", 22),
    ("BH", 22),
    ("BR", 29),
    ("CH", 21),
    ("CY", 28),
    ("CZ", 24),
    ("DE", 22),
    ("DK", 18),
    ("EE", 20),
    ("ES", 24),
    ("FI", 18),
    ("FO", 18),
    ("FR", 27),
    ("GB", 22),
    ("GI", 23),
    ("GL", 18),
    ("GR", 27),
    ("HR", 21),
    ("HU", 28),
    ("IE", 22),
    ("IL", 23),
    ("IS", 26),
    ("IT", 27),
    ("JO", 30),
    ("KW", 30),
    ("KZ", 20),
    ("LB", 28),
    ("LI", 21),
    ("LT", 20),
    ("LU", 20),
    ("LV", 21),
    ("MC", 27),
    ("MT", 31),
    ("NL", 18),
    ("NO", 15),
    ("PK", 24),
    ("PL", 28),
    ("PT", 25),
    ("QA", 29),
    ("RO", 24),
    ("RS", 22),
    ("SA", 24),
    ("SE", 24),
    ("SI", 19),
    ("SK", 24),
    ("SM", 27),
    ("TR", 26),
    ("UA", 29),
];

fn validate(s: &str) -> Option<f32> {
    let compact: String = s.chars().filter(|c| *c != ' ').collect();
    let expected = COUNTRY_LEN
        .iter()
        .find(|(cc, _)| compact.starts_with(cc))
        .map(|(_, len)| *len)?;
    (compact.len() == expected && iban_valid(&compact)).then_some(0.85)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("IBAN", PATTERN, 0.85)?
        .validator(validate)
        .context(&["iban", "账户", "account"], 0.1))
}
