mod support;

use support::{assert_none, assert_single, hits};

#[test]
fn ssn_hits() {
    assert_single("SSN 536-22-1234", "US_SSN", "536-22-1234");
    assert_single("social security: 536 22 1234", "US_SSN", "536 22 1234");
    assert_single("ssn 536221234", "US_SSN", "536221234");
}

#[test]
fn ssn_invalid_ranges_miss() {
    for s in [
        "666-12-3456",
        "900-12-3456",
        "000-12-3456",
        "536-00-1234",
        "536-22-0000",
        "078-05-1120",
    ] {
        assert_none(&format!("ssn {s}"));
    }
}

#[test]
fn ssn_lookalikes_miss() {
    assert_none("2026-11-26 order 536221234");
    assert_none("v536-22-1234x");
    assert!(hits("call 212-555-0147").iter().all(|(e, _)| e != "US_SSN"));
}

#[test]
fn us_phone_hits() {
    assert_single("(212) 555-0147", "US_PHONE", "(212) 555-0147");
    assert_single("tel +1 212-555-0147", "US_PHONE", "+1 212-555-0147");
    assert_single("phone 2125550147", "US_PHONE", "2125550147");
}

#[test]
fn us_phone_invalid_nanp_misses() {
    for p in [
        "(123) 555-0147",
        "(911) 555-0147",
        "(212) 155-0147",
        "(212) 911-0147",
    ] {
        assert_none(&format!("phone {p}"));
    }
}

#[test]
fn us_phone_lookalikes_miss() {
    assert_none("ts 1764123456");
    assert_none("latency_ns 2125550147");
}
