# piiguard-rs

面向 LLM 网关热路径的 PII 检测与可逆脱敏 Rust 库 + CLI（v0.1，尚未发布到 crates.io）。

- **低误报**：正则只负责找候选，再过校验位（Luhn、IBAN mod-97、GB 11643、GB 32100）、号段/日期/区划码校验，以及命中前 32 个字符内的上下文词加分。
- **可逆脱敏**：`Pseudonymize` 把 PII 换成 `<EMAIL_1>` 这样的占位符，映射存在 `Vault`；模型响应回来后用 `restore` 换回原值。
- **同步、无 async**：库只依赖 `regex`；`serde`/`serde_json` 是可选 feature，`clap` 只给 CLI 用。

## 安装

```toml
# 仅作库使用（不引入 clap）
piiguard = { path = "../piiguard-rs", default-features = false }
# 需要 JSON 配置 / Finding、Vault 序列化
piiguard = { path = "../piiguard-rs", default-features = false, features = ["serde"] }
```

默认 feature 是 `cli`（= `serde` + `clap`），这样 `cargo test` / `cargo clippy --all-targets` 会覆盖 CLI。

## 库用法

```rust
use piiguard::{Action, Analyzer, Vault, redact, restore};

let analyzer = Analyzer::builder()
    .with_builtin(&["EMAIL", "CN_ID_CARD", "CN_PHONE"]) // 或 .with_all_builtin()
    .threshold(0.5)                                     // 默认 0.5
    .allow(&["noreply@example.com"])                    // 允许列表：完全相同的命中不报
    .deny("CODENAME", &["凤凰计划"])                     // 拒绝列表：一律报出，score 1.0
    .build()?;

let text = "联系 alice@example.com";
let findings = analyzer.analyze(text); // 已去重叠、按位置排序、score ≥ 阈值

let mut vault = Vault::new();
let masked = redact(text, &findings, Action::Pseudonymize(&mut vault));
assert_eq!(masked, "联系 <EMAIL_1>");
assert_eq!(restore(&masked, &vault), text);
```

- `Finding { entity, start, end, char_start, char_end, score }`：`start/end` 是 UTF-8 字节偏移，`char_*` 是字符偏移。
- `Action::Mask { ch }`：逐字符替换，字符数不变。
- `Action::Replace`：`<ENTITY_n>`，一次调用内同值同号，不可还原。
- `Action::Pseudonymize(&mut Vault)`：同一 `Vault` 内同值始终同号，可还原。
- 自定义识别器：实现 `Recognizer` trait，或用 `PatternRecognizer::new(entity, pattern, score)?.validator(f).context(&words, boost).negative_context(&words, penalty)`，再 `.with_recognizer(r)`。

### 流式还原（SSE）

模型响应按 chunk 到达时，占位符可能被切成 `<EMA` + `IL_1>`。`StreamRestorer` 只扣住末尾"恰好是某个已知占位符前缀"的一小段，其余立即输出：

```rust
use piiguard::StreamRestorer;

let mut r = StreamRestorer::new(&vault);
let mut out = String::new();
for chunk in ["回复 <EMA", "IL_1>。"] {
    out.push_str(&r.push(chunk)); // 每个 chunk 立即返回可输出的部分
}
out.push_str(&r.finish());
assert_eq!(out, "回复 alice@example.com。");
```

任意切分下，所有输出拼起来都等于对完整文本调用 `restore`（`tests/stream.rs` 用 proptest 验证）。

## 内置识别器

| 实体 | 校验 | 无上下文基础分 | 上下文加分 |
|---|---|---|---|
| `EMAIL` | 结构（无 `..`、首尾点、域名标签不以 `-` 起止） | 0.9 | — |
| `IP_ADDRESS` | IPv4/IPv6 用标准库解析；IPv6 至少 3 个非空分组 | 0.6 | +0.3 |
| `CREDIT_CARD` | Luhn + 发卡组织号段与长度（Visa/MC/Amex/Discover/JCB） | 0.7 | +0.25 |
| `IBAN` | mod-97 + 国家固定长度（52 个常见国家） | 0.85 | +0.1 |
| `URL_TOKEN` | 查询参数名（`token`、`access_token`、`password`、`sig`…）且值 ≥ 6 字符 | 0.75 | — |
| `US_SSN` | 排除 000/666/9xx、组号 00、序号 0000、已作废示例号 | 带分隔符 0.6 / 纯 9 位 0.3 | +0.35 |
| `US_PHONE` | NANP：区号、交换码首位 2–9 且非 N11 | 有格式 0.5 / 纯 10 位 0.3 | +0.3 |
| `CN_ID_CARD` | 省级区划码 + 出生日期合法 + GB 11643 校验位 | 0.9 | +0.1 |
| `CN_PHONE` | 已分配号段前三位，支持 `+86`/`0086` 和空格/连字符分组 | 0.55 | +0.35 |
| `CN_USCC` | 登记管理部门码与机构类别码组合（GB 32100 表 1/2）+ 区划码前两位 + 校验位 | 0.85 | +0.1 |
| `CN_BANK_CARD` | 16–19 位 + Luhn；`62` 开头 0.45，其余 0.35 | 0.35–0.45 | +0.4 |

负向上下文：数字类识别器的命中紧跟在 `order`、`tracking`、`订单号`、`流水号` 等字段名之后时扣 0.5 分；IP 紧跟在 `ver`、`version`、`build`、`版本` 之后时扣 0.5 分（四段版本号在字面上就是合法 IPv4）。只看紧挨着的那个完整单词，所以 `server 10.0.0.1` 不受影响。

`CN_BANK_CARD`、纯数字 `US_SSN`/`US_PHONE` 的基础分刻意低于默认阈值：随机数字串有 1/10 概率通过 Luhn，没有"银行卡/卡号/ssn/phone"等上下文词时不报。

## CLI

```bash
cargo build --release
```

```bash
echo '邮件 a@example.com，手机 13812345678' | target/release/piiguard scan
```

```bash
target/release/piiguard redact --action pseudonymize --vault vault.json input.txt > redacted.txt
```

```bash
target/release/piiguard restore --vault vault.json redacted.txt
```

- `scan`：输出 JSON `{count, findings}`，**不包含命中原文**。
- `redact --action mask|replace|pseudonymize [--mask-char '*']`：输出处理后的文本；`pseudonymize` 必须给 `--vault`，文件已存在则追加。
- 通用参数：`[FILE]`（省略读 stdin）、`--config cfg.json`、`--threshold`、`--entities EMAIL,CN_PHONE`（覆盖配置）。

`vault.json` 里是**明文原值**，请按敏感文件处理。

配置文件（`serde` feature，未知字段报错）：

```json
{
  "entities": ["EMAIL", "CN_ID_CARD"],
  "threshold": 0.6,
  "allow": ["noreply@example.com"],
  "deny": [{ "entity": "CODENAME", "values": ["凤凰计划"] }],
  "custom": [{ "entity": "EMPLOYEE_ID", "pattern": "EMP-[0-9]{6}", "score": 0.5,
               "context": ["工号"], "context_boost": 0.2 }]
}
```

省略 `entities` 表示启用全部内置识别器，`[]` 表示一个都不启用。

## 测试与基准

```bash
cargo test
```

```bash
cargo clippy --all-targets -- -D warnings
```

```bash
cargo bench --bench analyze
```

- 测试样本全部由 `tests/support/mod.rs` 里的算法生成（与库里的校验实现分开写），不含真实个人信息。
- 每个识别器都有三类测试：合法样本命中、校验位/结构错误不命中、相似但非 PII（订单号、时间戳、版本号等）不命中。
- `tests/roundtrip.rs` 用 proptest 验证 `restore(redact(x, Pseudonymize)) == x`（512 例，含任意 Unicode、合成 PII、字面 `<EMAIL_1>`）。
- `tests/false_positive.rs`：在本仓库 `Cargo.lock` + 3000 行合成网关日志上，默认阈值命中数为 0。这只是一个小基线，不代表真实流量上的误报率。
- `benches/analyze.rs` 是 criterion 微基准。与 presidio 等 9 个工具的同语料对比见 [docs/benchmark.md](docs/benchmark.md)（`bench/run.sh` 一条命令复现），功能对比见 [docs/comparison.md](docs/comparison.md)。

## 限制（v0.1）

- 不做密钥/API key 格式识别（`sk-`、`ghp_`、`AKIA` 等），也不做 NER（人名、地址）；`Recognizer` trait 留作扩展点。
- `URL_TOKEN` 只按参数名识别，参数名不在列表里就不会报。
- 不带 `ver`/`version`/`版本` 等字段名的四段版本号仍会被当成 IPv4（如 `1.2.3.4.5` 中的 `1.2.3.4`）；SSN、电话号码没有校验位，只能靠号段规则和上下文词。
- 正则是最左优先、不回溯到校验函数：候选串如果被贪婪匹配吞进了相邻内容（如带空格的 IBAN 后面紧跟大写单词），校验失败后不会再尝试更短的子串，结果是漏报。
- 上下文词只看命中**之前** 32 个字符。
- `Vault` 只在内存里；新分配占位符时会跳过当前原文里已出现的同名占位符，但如果同一个 `Vault` 跨多段文本复用，后来的文本里字面出现的旧占位符会在 `restore` 时被替换。
- 不做任何合规认证声明。
