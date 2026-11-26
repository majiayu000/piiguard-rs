use std::net::{Ipv4Addr, Ipv6Addr};

use super::PatternRecognizer;

const PATTERN: &str = concat!(
    r"[0-9]{1,3}(?:\.[0-9]{1,3}){3}",
    r"|(?:[0-9A-Fa-f]{0,4}:){2,7}[0-9A-Fa-f]{0,4}",
);

const CONTEXT: &[&str] = &["ip", "addr", "地址", "host", "client", "remote"];

/// 四段版本号（`ver=1.2.3.4`）在字面上就是合法 IPv4，只能靠字段名区分。
const VERSION_WORDS: &[&str] = &["ver", "version", "build", "release", "版本", "版本号"];

/// IPv4 交给标准库解析（拒绝 >255 与前导零）；IPv6 至少 3 个非空分组，排除 `::1` 这类短串。
fn validate(s: &str) -> Option<f32> {
    if s.contains('.') {
        return s.parse::<Ipv4Addr>().is_ok().then_some(0.6);
    }
    let groups = s.split(':').filter(|g| !g.is_empty()).count();
    (groups >= 3 && s.parse::<Ipv6Addr>().is_ok()).then_some(0.6)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("IP_ADDRESS", PATTERN, 0.6)?
        .validator(validate)
        .context(CONTEXT, 0.3)
        .negative_context(VERSION_WORDS, 0.5))
}
