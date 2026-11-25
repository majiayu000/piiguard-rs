//! 合成号码生成器。刻意与 `piiguard::checksum` 分开独立实现，避免"用同一份代码生成又校验"。
//! 所有号码都由算法拼出，不对应任何已知真实个人或机构。
#![allow(dead_code)]

/// 给数字前缀补上 Luhn 校验位。
pub fn luhn_complete(prefix: &str) -> String {
    let mut sum = 0;
    for (i, c) in prefix.chars().rev().enumerate() {
        let mut d = c.to_digit(10).unwrap();
        if i % 2 == 0 {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }
    format!("{prefix}{}", (10 - sum % 10) % 10)
}

/// 把一个合法号码的最后一位换成别的数字，得到校验位错误的号码。
pub fn corrupt_last(valid: &str) -> String {
    let (head, last) = valid.split_at(valid.len() - 1);
    let next = match last {
        "9" | "X" | "x" | "Y" => "0".to_string(),
        d => (d.parse::<u32>().map(|n| n + 1).unwrap_or(0)).to_string(),
    };
    format!("{head}{next}")
}

/// 18 位身份证：6 位地区码 + 8 位生日 + 3 位顺序码 + GB 11643 校验位。
pub fn cn_id(region: &str, birth: &str, seq: &str) -> String {
    let body = format!("{region}{birth}{seq}");
    let w = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
    let s: u32 = body
        .chars()
        .zip(w)
        .map(|(c, w)| c.to_digit(10).unwrap() * w)
        .sum();
    let check = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'][(s % 11) as usize];
    format!("{body}{check}")
}

/// 统一社会信用代码：17 位本体 + GB 32100 校验位。
pub fn cn_uscc(body17: &str) -> String {
    let cs = "0123456789ABCDEFGHJKLMNPQRTUWXY";
    let w = [
        1, 3, 9, 27, 19, 26, 16, 17, 20, 29, 25, 13, 8, 24, 10, 30, 28,
    ];
    let s: usize = body17
        .chars()
        .zip(w)
        .map(|(c, w)| cs.find(c).unwrap() * w)
        .sum();
    let check = (31 - s % 31) % 31;
    format!("{body17}{}", cs.as_bytes()[check] as char)
}

/// IBAN：国家码 + 两位校验数字（98 - mod97）+ BBAN。
pub fn iban(country: &str, bban: &str) -> String {
    let digits: String = format!("{bban}{country}00")
        .chars()
        .map(|c| c.to_digit(36).unwrap().to_string())
        .collect();
    let rem = digits
        .chars()
        .fold(0u32, |r, c| (r * 10 + c.to_digit(10).unwrap()) % 97);
    format!("{country}{:02}{bban}", 98 - rem)
}

pub fn visa() -> String {
    luhn_complete("411111111111111")
}

pub fn id_card() -> String {
    cn_id("110101", "19900307", "123")
}

pub fn uscc() -> String {
    cn_uscc("91110108MA01ABCD1")
}

pub fn unionpay() -> String {
    luhn_complete("621700000000000000")
}

/// 用全部内置识别器、默认阈值扫描，返回 (实体, 命中原文)。
pub fn hits(text: &str) -> Vec<(String, String)> {
    let a = piiguard::Analyzer::builder()
        .with_all_builtin()
        .build()
        .unwrap();
    a.analyze(text)
        .into_iter()
        .map(|f| (f.entity.clone(), text[f.start..f.end].to_string()))
        .collect()
}

/// 断言文本恰好命中一个 `entity`，原文为 `value`。
pub fn assert_single(text: &str, entity: &str, value: &str) {
    assert_eq!(
        hits(text),
        vec![(entity.to_string(), value.to_string())],
        "text: {text}"
    );
}

pub fn assert_none(text: &str) {
    assert_eq!(hits(text), vec![], "text: {text}");
}
