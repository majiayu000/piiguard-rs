mod support;

use support::{assert_none, assert_single, corrupt_last, iban, luhn_complete, visa};

#[test]
fn email_hits() {
    assert_single(
        "联系 alice.w@example.com 谢谢",
        "EMAIL",
        "alice.w@example.com",
    );
    assert_single(
        "邮箱bob+tag@mail.example.org。",
        "EMAIL",
        "bob+tag@mail.example.org",
    );
}

#[test]
fn email_invalid_structure_misses() {
    assert_none("a..b@example.com");
    assert_none(".a@example.com");
    assert_none("user@localhost");
}

#[test]
fn email_lookalikes_miss() {
    assert_none("npm i @types/node && pip install foo==1.2");
    assert_none("2026-11-26T12:30:45Z order#20261126001 @channel");
}

#[test]
fn ip_hits() {
    assert_single(
        "client 192.168.1.20 connected",
        "IP_ADDRESS",
        "192.168.1.20",
    );
    assert_single(
        "remote=2001:db8:85a3::8a2e:370:7334;",
        "IP_ADDRESS",
        "2001:db8:85a3::8a2e:370:7334",
    );
}

#[test]
fn ip_invalid_octets_miss() {
    assert_none("256.1.1.1");
    assert_none("10.0.0.999");
    assert_none("01.2.3.4");
}

#[test]
fn ip_lookalikes_miss() {
    assert_none("2026-11-26 12:30:45 started");
    assert_none("mac aa:bb:cc:dd:ee:ff");
    assert_none("use std::io::Result; listen on ::1");
}

#[test]
fn credit_card_hits() {
    let v = visa();
    assert_single(&format!("card {v}"), "CREDIT_CARD", &v);
    let spaced = format!("{} {} {} {}", &v[..4], &v[4..8], &v[8..12], &v[12..]);
    assert_single(&format!("卡号：{spaced}。"), "CREDIT_CARD", &spaced);
    let amex = luhn_complete("37828224631000");
    assert_single(&format!("amex {amex}"), "CREDIT_CARD", &amex);
}

#[test]
fn credit_card_bad_checksum_misses() {
    assert_none(&format!("card {}", corrupt_last(&visa())));
}

#[test]
fn credit_card_lookalikes_miss() {
    assert_none("ts=1764123456789 order=20261126123456789012");
    assert_none("trace_id 202611261230450001");
}

#[test]
fn iban_hits() {
    let i = iban("DE", "370400440532013000");
    assert_single(&format!("IBAN {i}"), "IBAN", &i);
    let spaced = format!(
        "{} {} {} {} {} {}",
        &i[..4],
        &i[4..8],
        &i[8..12],
        &i[12..16],
        &i[16..20],
        &i[20..]
    );
    assert_single(&format!("pay to {spaced}."), "IBAN", &spaced);
}

#[test]
fn iban_bad_checksum_misses() {
    assert_none(&corrupt_last(&iban("DE", "370400440532013000")));
    assert_none(&corrupt_last(&iban("GB", "WEST12345698765432")));
}

#[test]
fn iban_lookalikes_miss() {
    assert_none("build DE12345678901234567890 done");
    assert_none("sku XX12ABCDEFGHIJKLMNOP");
}

#[test]
fn url_token_hits() {
    let url = "https://api.example.net/cb?code=1&access_token=abcDEF123456&x=1";
    assert_single(url, "URL_TOKEN", "abcDEF123456");
    assert_single(
        "GET /login?Password=hunter2hunter2 HTTP/1.1",
        "URL_TOKEN",
        "hunter2hunter2",
    );
}

#[test]
fn url_token_short_value_misses() {
    assert_none("/feed?token=1&page=2");
}

#[test]
fn url_token_lookalikes_miss() {
    assert_none("/search?page=2&sort=desc&tokenizer=bpe&max_tokens=1024");
}

#[test]
fn ip_after_version_field_is_suppressed() {
    assert_none("ver=2.10.7.180 path=/v1");
    assert_none("版本 9.4.8.67 已发布");
    assert_single("server 10.2.3.4 down", "IP_ADDRESS", "10.2.3.4");
}

#[test]
fn card_after_order_field_is_suppressed() {
    let v = visa();
    assert_none(&format!("order={v} latency_ms=3"));
    assert_none(&format!("订单号 {v}"));
}
