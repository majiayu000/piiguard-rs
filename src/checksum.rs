//! 校验位算法：Luhn、IBAN mod-97、GB 11643（身份证）、GB 32100（统一社会信用代码）。
//!
//! 所有函数只接受已去掉分隔符的输入，遇到非法字符返回 `false`。

/// Luhn 校验（信用卡、银行卡）。输入必须全是 ASCII 数字。
pub fn luhn_valid(digits: &str) -> bool {
    if digits.len() < 2 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let sum: u32 = digits
        .bytes()
        .rev()
        .enumerate()
        .map(|(i, b)| {
            let d = u32::from(b - b'0');
            if i % 2 == 1 {
                let x = d * 2;
                if x > 9 { x - 9 } else { x }
            } else {
                d
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

/// IBAN mod-97 校验（ISO 13616）：前 4 位挪到末尾、字母转两位数字后对 97 取模应为 1。
/// 输入为去掉空格的大写字母数字串。
pub fn iban_valid(iban: &str) -> bool {
    let bytes = iban.as_bytes();
    if bytes.len() < 5
        || !bytes
            .iter()
            .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase())
    {
        return false;
    }
    let mut rem: u32 = 0;
    for &b in bytes[4..].iter().chain(&bytes[..4]) {
        if b.is_ascii_digit() {
            rem = (rem * 10 + u32::from(b - b'0')) % 97;
        } else {
            rem = (rem * 100 + u32::from(b - b'A') + 10) % 97;
        }
    }
    rem == 1
}

const GB11643_WEIGHTS: [u32; 17] = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
const GB11643_CHECK: &[u8; 11] = b"10X98765432";

/// 根据身份证前 17 位计算 GB 11643 校验码（`0-9` 或 `X`）。前 17 位必须全是数字。
pub fn gb11643_check_char(first17: &str) -> Option<char> {
    let b = first17.as_bytes();
    if b.len() != 17 || !b.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let sum: u32 = b
        .iter()
        .zip(GB11643_WEIGHTS)
        .map(|(&d, w)| u32::from(d - b'0') * w)
        .sum();
    Some(char::from(GB11643_CHECK[(sum % 11) as usize]))
}

/// 18 位身份证号的 GB 11643 校验（`x` 视同 `X`）。
pub fn gb11643_valid(id: &str) -> bool {
    if id.len() != 18 || !id.is_ascii() {
        return false;
    }
    let last = id.as_bytes()[17].to_ascii_uppercase();
    gb11643_check_char(&id[..17]) == Some(char::from(last))
}

/// GB 32100 字符集：数字 + 去掉 I、O、Z、S、V 的大写字母，共 31 个。
pub const GB32100_CHARSET: &[u8; 31] = b"0123456789ABCDEFGHJKLMNPQRTUWXY";

fn gb32100_value(c: u8) -> Option<u32> {
    GB32100_CHARSET
        .iter()
        .position(|&x| x == c)
        .map(|p| p as u32)
}

/// 根据统一社会信用代码前 17 位计算 GB 32100 校验码。权重为 3^i mod 31。
pub fn gb32100_check_char(first17: &str) -> Option<char> {
    let b = first17.as_bytes();
    if b.len() != 17 {
        return None;
    }
    let mut sum = 0u32;
    let mut weight = 1u32;
    for &c in b {
        sum += gb32100_value(c)? * weight;
        weight = weight * 3 % 31;
    }
    let check = (31 - sum % 31) % 31;
    Some(char::from(GB32100_CHARSET[check as usize]))
}

/// 18 位统一社会信用代码的 GB 32100 校验。输入应为大写。
pub fn gb32100_valid(code: &str) -> bool {
    if code.len() != 18 || !code.is_ascii() {
        return false;
    }
    gb32100_check_char(&code[..17]) == Some(char::from(code.as_bytes()[17]))
}

#[cfg(test)]
mod tests {
    use super::*;

    // 以下均为教科书/标准文本中的示例号码，不对应真实个人。
    #[test]
    fn luhn_textbook_example() {
        assert!(luhn_valid("79927398713"));
        assert!(!luhn_valid("79927398710"));
        assert!(!luhn_valid("7992739871a"));
    }

    #[test]
    fn iban_iso13616_example() {
        assert!(iban_valid("GB82WEST12345698765432"));
        assert!(!iban_valid("GB83WEST12345698765432"));
    }

    #[test]
    fn gb11643_standard_example() {
        assert!(gb11643_valid("11010519491231002X"));
        assert!(gb11643_valid("11010519491231002x"));
        assert!(!gb11643_valid("110105194912310021"));
    }

    #[test]
    fn gb32100_rejects_illegal_chars() {
        assert_eq!(gb32100_check_char("9111000000000000I"), None);
        assert!(!gb32100_valid("91110000000000000"));
    }
}
