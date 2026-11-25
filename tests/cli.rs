#![cfg(feature = "cli")]

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run(args: &[&str], stdin: &str) -> (bool, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_piiguard"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap())
}

fn tmp(name: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_file(&p);
    p
}

const TEXT: &str = "邮件 a@example.com，手机 13812345678\n";

#[test]
fn scan_outputs_json_without_values() {
    let (ok, out) = run(&["scan"], TEXT);
    assert!(ok);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["count"], 2);
    assert_eq!(v["findings"][0]["entity"], "EMAIL");
    assert!(!out.contains("a@example.com"));
}

#[test]
fn scan_respects_entities_flag() {
    let (_, out) = run(&["scan", "--entities", "CN_PHONE"], TEXT);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["count"], 1);
}

#[test]
fn redact_mask_and_replace() {
    assert_eq!(
        run(&["redact", "--action", "replace"], TEXT).1,
        "邮件 <EMAIL_1>，手机 <CN_PHONE_1>\n"
    );
    let masked = run(&["redact", "--action", "mask", "--mask-char", "#"], TEXT).1;
    assert_eq!(masked, "邮件 #############，手机 ###########\n");
}

#[test]
fn pseudonymize_then_restore_roundtrips() {
    let vault = tmp("cli_vault.json");
    let v = vault.to_str().unwrap();
    let (ok, redacted) = run(&["redact", "--action", "pseudonymize", "--vault", v], TEXT);
    assert!(ok);
    assert!(!redacted.contains("13812345678"));
    let (ok, restored) = run(&["restore", "--vault", v], &redacted);
    assert!(ok);
    assert_eq!(restored, TEXT);
}

#[test]
fn pseudonymize_without_vault_fails() {
    assert!(!run(&["redact", "--action", "pseudonymize"], TEXT).0);
}
