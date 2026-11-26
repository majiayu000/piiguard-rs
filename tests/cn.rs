mod support;

use support::{assert_none, assert_single, cn_id, corrupt_last, hits, id_card, unionpay, uscc};

#[test]
fn id_card_hits_even_glued_to_chinese() {
    let id = id_card();
    assert_single(&format!("身份证{id}，请核对"), "CN_ID_CARD", &id);
    assert_single(&format!("证件号: {id}"), "CN_ID_CARD", &id);
}

#[test]
fn id_card_bad_checksum_misses() {
    assert_none(&format!("身份证 {}", corrupt_last(&id_card())));
}

#[test]
fn id_card_invalid_date_or_region_misses() {
    assert_none(&cn_id("110101", "19900230", "123"));
    assert_none(&cn_id("990101", "19900307", "123"));
}

#[test]
fn id_card_lookalikes_miss() {
    assert_none("订单号 202611261230450001");
    assert_none("时间戳 20261126123045 批次 2026112612304500");
}

#[test]
fn cn_phone_hits() {
    assert_single("手机13812345678", "CN_PHONE", "13812345678");
    assert_single("tel: +86 138 1234 5678", "CN_PHONE", "+86 138 1234 5678");
    assert_single("电话：138-1234-5678。", "CN_PHONE", "138-1234-5678");
}

#[test]
fn cn_phone_invalid_segment_misses() {
    assert_none("手机 14012345678");
    assert_none("手机 12012345678");
}

#[test]
fn cn_phone_lookalikes_miss() {
    assert_none("ts=1381234567890");
    assert_none("订单 13812345678901");
}

#[test]
fn uscc_hits() {
    let c = uscc();
    assert_single(&format!("统一社会信用代码：{c}"), "CN_USCC", &c);
}

#[test]
fn uscc_bad_checksum_misses() {
    assert_none(&format!("信用代码 {}", corrupt_last(&uscc())));
}

#[test]
fn uscc_lookalikes_miss() {
    assert_none("image tag 9111010800000000AB");
    assert_none("订单号 202611261230450001");
}

#[test]
fn bank_card_hits_with_context() {
    let c = unionpay();
    assert_single(&format!("银行卡号 {c}"), "CN_BANK_CARD", &c);
}

#[test]
fn bank_card_without_context_stays_below_default_threshold() {
    assert_none(&unionpay());
}

#[test]
fn bank_card_bad_checksum_misses() {
    assert_none(&format!("银行卡号 {}", corrupt_last(&unionpay())));
}

#[test]
fn bank_card_lookalikes_miss() {
    assert_none("订单号 6217000000000000001 已发货");
    assert!(hits("流水 20261126123456789").is_empty());
}

#[test]
fn uscc_with_invalid_dept_category_misses() {
    // "94" 不是 GB 32100 允许的登记管理部门码 + 机构类别码组合，即使校验位正确也不报
    assert_none(&support::cn_uscc("94110108MA01ABCD1"));
}

#[test]
fn bank_card_transfer_context_hits() {
    let c = unionpay();
    assert_single(&format!("转账至 {c} 即可"), "CN_BANK_CARD", &c);
    assert_single(&format!("对公账户 {c}"), "CN_BANK_CARD", &c);
}
