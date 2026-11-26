# 基准：piiguard 与同类工具的吞吐、延迟、内存、准确率（2026-11-26）

> 本文所有数字都来自 `bench/results/before/` 与 `bench/results/after/` 下的原始文件（`summary.json`、`summary.md`、`raw/`、`env.txt`）。每组只跑了一次，没有多次取均值；其他工具在 before/after 两次运行之间没有任何改动，吞吐却相差约 5–30%（例如 pii-veil labeled 1.98 → 2.55、llm-pii-redact 10.73 → 11.73），redacter 的进程墙钟相差约 70%（0.34 → 0.58），比较时请考虑这个噪声。

## 1. 环境

| 项 | 值（`env.txt`） |
|---|---|
| 机器 | MacBookPro18,3 / Apple M1 Pro / 8 核 / 32 GB |
| 系统 | macOS 26.6.2 |
| Rust | rustc 1.100.0，release 构建 |
| Python | 3.12.12（uv 管理的 venv） |
| Rust 竞品 | presidio-analyzer 0.1.11、pii-vault 0.2.0、pii-veil-core 1.0.0、cloakpipe-core 0.15.0、llm-pii-redact 0.1.0、redacter 0.23.0、regex 1.13.2 |
| Python 竞品 | presidio-analyzer / presidio-anonymizer 2.2.366、scrubadub 2.0.1、spacy 3.8.18、en-core-web-lg 3.8.0、phonenumbers 9.0.44 |
| piiguard | before = git `ee72718`；after = `ee72718` + 本期未提交改动 |

各工具配置：

| 工具 | 配置 |
|---|---|
| piiguard | `with_all_builtin()`，默认阈值 0.5 |
| presidio-spacy | `AnalyzerEngine()` 默认：SpacyNlpEngine(`en_core_web_lg`) + 默认英文识别器，阈值 0 |
| presidio-rules | `NoOpNlpEngine` + 默认英文识别器去掉 `SpacyRecognizer`，阈值 0（没有 NLP 产物，词形上下文增强不生效） |
| presidio-rs | `AnalyzerEngine::new()` 默认，阈值 0 |
| pii-vault | crate 包里不带规则，使用同版本 git tag v0.2.0 的 `spec/recognizers/`（47 个 JSON），阈值 0 |
| pii-veil | `DetectorPipeline::new()` 默认 |
| cloakpipe | `DetectionConfig` 默认值 + 打开 `phone_numbers`、`ip_addresses`（工具本身支持，但默认关闭） |
| llm-pii-redact | `Redactor::default()` |
| redacter | `redacter scan -d local-rules`（CLI，每篇文档一个文件） |
| scrubadub | `Scrubber()` 默认检测器 |

## 2. 语料

由 `bench/python/gen_corpus.py` 生成，随机种子 `SEED = 20261126`。PII 全部由 `bench/python/gen_pii.py` 按算法生成：邮箱只用 example.com/org/net，IP 只用 RFC 5737/3849 文档地址段和私有段，美国电话只用 555-0100..0199。

| 语料 | 篇数 | 文本字节（UTF-8） | 内容 |
|---|---|---|---|
| labeled | 4000 | 550,519 | 中英文模板句，每篇 1–4 句；标注 9 类 PII 共 9,031 处。另含硬负样本：订单号、时间戳、四段版本号、请求 ID，以及校验位被改坏的卡号、身份证、IBAN、信用代码（这些都不标注） |
| clean | 2517 | 3,503,286 | 1500 篇合成网关日志（时间戳、订单号、版本号、hex 请求 ID）+ 1017 段真实 Rust 源码（`bench/rust/Cargo.lock` 锁定的 serde_json 1.0.154、aho-corasick 1.1.5 的 `src/`，每 40 行一段；已 grep 确认没有邮箱） |

金标各类数量：EMAIL 1531、PHONE 2014（CN 1234 + US 780）、CN_BANK_CARD 1230、IP_ADDRESS 1188、CN_USCC 830、US_SSN 773、CREDIT_CARD 681、IBAN 393、CN_ID_CARD 391。

**已知缺陷**：clean 语料里有 7 段代码的 id 重复（不同目录下的同名文件，id 只取了上一级目录名），`score.py` 按 id 建字典时被合并，所以**误报统计实际覆盖 2510 篇**，吞吐和延迟仍按全部 2517 篇计算。修正需要改 id 规则并重跑，本轮没有重跑。

## 3. 指标定义

- **吞吐 MB/s** = 语料文本总字节 / 各篇检测调用耗时之和。只计检测函数本身，不含读文件和 JSON 解析；单线程；先对前 50 篇预热。redacter 只有 CLI，它的吞吐是整个进程的墙钟时间（含进程启动和逐文件 I/O），**与其他工具不可直接比较**，也没有逐篇延迟。
- **p50 / p99** = labeled 语料上单篇检测耗时的分位数（µs）。
- **内存峰值** = `/usr/bin/time -l` 报告的 runner 进程 maximum resident set size。包含把两份语料读进内存的开销（各工具相同），Python 工具还包含解释器和模型。
- **precision / recall / F1**：预测区间与金标区间有重叠且类型兼容就算命中，每个金标和每个预测最多匹配一次。
  - 类型映射见 `bench/python/labels.py`：中美电话合并为 PHONE；CREDIT_CARD 与 CN_BANK_CARD 互相兼容；pii-veil 的 NATIONAL_ID 按 US_SSN 计。
  - 工具不支持的类型记为"不支持"，不进入它的 micro 平均。
  - 不在评测范围内的预测类型（DATE_TIME、URL、PERSON 等）不计入 precision。
- **误报** = clean 语料上的全部预测。分两列：所有类型的预测数，以及只算 9 个评测类型的预测数。
- **panic 篇数**：runner 用 `catch_unwind` 捕获，panic 的那一篇记为无输出。

## 4. 全部工具结果（after 组）

### 4.1 性能

| 工具 | 吞吐 labeled MB/s | 吞吐 clean MB/s | p50 µs | p99 µs | 内存峰值 MB | 加载 ms | panic 篇数 |
|---|---|---|---|---|---|---|---|
| **piiguard** | **19.8** | 51.74 | **6.5** | **18.3** | **23.4** | 4 | 0 |
| pii-veil | 2.55 | **132.25** | 51.9 | 146.6 | 30.3 | 0 | 0 |
| llm-pii-redact | 11.73 | 67.14 | 11.2 | 30.0 | 24.8 | 2 | 0 |
| cloakpipe | 7.45 | 62.98 | 18.2 | 45.6 | 27.1 | 4 | 0 |
| pii-vault | 1.61 | 9.08 | 72.8 | 239.7 | 43.3 | 22 | **1150** |
| presidio-rs | 0.52 | 1.54 | 237.9 | 738.8 | 140.7 | 13 | 0 |
| scrubadub | 0.8 | 1.24 | 153.9 | 471.8 | 220.0 | 3191 | 0 |
| presidio-rules | 0.09 | 0.16 | 1338.1 | 3515.3 | 139.2 | 1794 | 0 |
| presidio-spacy | 0.03 | 0.05 | 4536.4 | 9104.8 | 891.7 | 2288 | 0 |
| redacter（进程墙钟） | 0.58 | 4.51 | — | — | 44.3 | — | 0 |

pii-vault 0.2.0 在 labeled 语料的 1150 篇上 panic，位置在 `pii-vault-0.2.0/src/recognizer.rs:63`：上下文窗口按字节切片，越过了中文字符边界。

### 4.2 准确率（labeled，P / R / F1）

| 工具 | EMAIL | CREDIT_CARD | IBAN | IP_ADDRESS | US_SSN | PHONE | CN_ID_CARD | CN_USCC | CN_BANK_CARD | micro P / R / F1 |
|---|---|---|---|---|---|---|---|---|---|---|
| **piiguard** | 1.00 / 1.00 / 1.00 | 1.00 / 1.00 / 1.00 | 1.00 / 1.00 / 1.00 | 1.00 / 1.00 / 1.00 | 1.00 / 0.75 / 0.86 | 1.00 / 1.00 / 1.00 | 1.00 / 1.00 / 1.00 | 1.00 / 1.00 / 1.00 | 0.99 / 1.00 / 1.00 | **0.999 / 0.978 / 0.988** |
| presidio-rules | 1.00 / 0.92 / 0.96 | 0.92 / 1.00 / 0.96 | 1.00 / 1.00 / 1.00 | 0.94 / 1.00 / 0.97 | 1.00 / 1.00 / 1.00 | 0.71 / 0.88 / 0.79 | 不支持 | 不支持 | 不支持 | 0.881 / 0.945 / 0.912 |
| presidio-spacy | 1.00 / 0.92 / 0.96 | 0.92 / 1.00 / 0.96 | 1.00 / 1.00 / 1.00 | 0.94 / 1.00 / 0.97 | 1.00 / 1.00 / 1.00 | 0.71 / 0.88 / 0.79 | 不支持 | 不支持 | 不支持 | 0.881 / 0.945 / 0.912 |
| llm-pii-redact | 1.00 / 1.00 / 1.00 | 0.85 / 1.00 / 0.92 | 0.53 / 0.51 / 0.52 | 0.94 / 1.00 / 0.97 | 1.00 / 1.00 / 1.00 | 0.69 / 0.68 / 0.69 | 不支持 | 不支持 | 不支持 | 0.851 / 0.873 / 0.862 |
| redacter | 1.00 / 1.00 / 1.00 | 0.92 / 1.00 / 0.96 | 1.00 / 1.00 / 1.00 | 0.94 / 1.00 / 0.97 | 1.00 / 0.50 / 0.67 | 0.89 / 0.42 / 0.57 | 1.00 / 0.91 / 0.95 | 不支持 | 不支持 | 0.953 / 0.772 / 0.853 |
| pii-veil | 1.00 / 1.00 / 1.00 | 0.98 / 1.00 / 0.99 | 1.00 / 1.00 / 1.00 | 0.97 / 1.00 / 0.98 | 0.83 / 0.50 / 0.63 | 0.90 / 0.30 / 0.45 | 不支持 | 不支持 | 不支持 | 0.962 / 0.726 / 0.827 |
| scrubadub | 0.86 / 1.00 / 0.92 | 0.83 / 0.38 / 0.52 | 不支持 | 不支持 | 1.00 / 0.50 / 0.67 | 0.98 / 0.77 / 0.86 | 不支持 | 不支持 | 不支持 | 0.917 / 0.743 / 0.821 |
| presidio-rs | 1.00 / 1.00 / 1.00 | 0.88 / 1.00 / 0.93 | 0.53 / 0.02 / 0.04 | 0.92 / 0.76 / 0.84 | 1.00 / 0.67 / 0.80 | 0.38 / 0.87 / 0.53 | 0.00 / 0.00 / 0.00 | 不支持 | 不支持 | 0.646 / 0.773 / 0.704 |
| cloakpipe | 1.00 / 1.00 / 1.00 | 不支持 | 不支持 | 0.92 / 0.76 / 0.84 | 1.00 / 0.50 / 0.66 | 0.23 / 1.00 / 0.37 | 不支持 | 不支持 | 不支持 | 0.410 / 0.878 / 0.559 |
| pii-vault | 1.00 / 0.59 / 0.74 | 0.92 / 0.56 / 0.69 | 1.00 / 0.55 / 0.71 | 0.93 / 0.41 / 0.57 | 1.00 / 0.39 / 0.56 | 0.19 / 0.56 / 0.28 | 0.00 / 0.00 / 0.00 | 不支持 | 0.77 / 0.25 / 0.38 | 0.429 / 0.454 / 0.441 |

说明：
- pii-vault 的召回偏低，主要是因为 panic 的那些文档没有输出。
- presidio-spacy 和 presidio-rules 在这些规则类实体上数字完全相同。spaCy NER 只额外产出 PERSON / LOCATION / NRP 等类型，不在评测范围内。
- presidio-rs 和 pii-vault 的 CN_ID_CARD 都是 0.00：所有中国身份证样本都没被它们以身份证类型识别出来。原因没有深入追查，推断与基础分、边界或标签有关。
- **piiguard 的 SSN 召回只有 0.75**：不带分隔符、也没有上下文词的 9 位纯数字 SSN 刻意不报（v0.1 的低误报设计），而 presidio 和 llm-pii-redact 在这一项是 1.00。

### 4.3 误报（clean 语料，2510 篇）

| 工具 | 误报（全部类型） | 误报（评测类型） | 主要标签 |
|---|---|---|---|
| **piiguard** | **10** | **10** | CREDIT_CARD 9, IP_ADDRESS 1 |
| scrubadub | 440 | 338 | phone 322, twitter 55, url 47, email 10, credit_card 6 |
| pii-veil | 1537 | 1537 | IPV4 1435, CREDIT_CARD 102 |
| redacter | 2043 | 2012 | ipv4 1435, payment-card 424, ipv6 141, austrian-svnr 13, phone-international 12, swedish-personnummer 9 |
| llm-pii-redact | 8224 | 8177 | PHONE_US 4996, CREDIT_CARD 1465, IP_V4 1435, SSN 259, URL 47, IP_V6 22 |
| presidio-rs | 34576 | 8911 | DATE_TIME 15276, US_BANK_NUMBER 7802, PHONE_NUMBER 6624, IP_ADDRESS 1435, VIN 743, CREDIT_CARD 701 |
| pii-vault | 28928 | 27414 | PHONE_NUMBER 24754, IP_ADDRESS 1434, CRYPTO 1396, CN_BANK_CARD 545, CREDIT_CARD 541, CN_ID_CARD 140 |
| cloakpipe | 34732 | 23550 | PhoneNumber 22115, Date 11173, IpAddress 1435, Percentage 9 |
| presidio-rules | 70382 | 5170 | US_DRIVER_LICENSE 30543, DATE_TIME 15368, US_BANK_NUMBER 15063, URL 3385, PHONE_NUMBER 3160, IP_ADDRESS 1457 |
| presidio-spacy | 89871 | 5170 | DATE_TIME 31674, US_DRIVER_LICENSE 30543, US_BANK_NUMBER 15063, URL 3385, PHONE_NUMBER 3160, PERSON 1856 |

presidio 系列、pii-vault 都按各自默认阈值 0 运行，所以低分候选也全部计入；提高阈值会减少误报，本轮没有测阈值曲线。

## 5. piiguard 优化前后（同一套语料与脚本）

| 指标 | before（ee72718） | after | 变化来源 |
|---|---|---|---|
| 吞吐 labeled MB/s | 1.68 | 19.8 | 去掉 `RegexSet` 预筛 |
| 吞吐 clean MB/s | 12.77 | 51.74 | 同上 |
| p50 / p99 µs（labeled） | 49.2 / 182.6 | 6.5 / 18.3 | 同上 |
| p50 / p99 µs（clean） | 83.4 / 561.0 | 22.7 / 54.3 | 同上 |
| 内存峰值 MB | 26.5 | 23.4 | — |
| micro P / R / F1 | 0.988 / 0.890 / 0.936 | 0.999 / 0.978 / 0.988 | 见下 |
| IP_ADDRESS P | 0.941 | 1.000 | 负向上下文（`ver`/`version`/`版本`…） |
| CREDIT_CARD P | 0.990 | 1.000 | 负向上下文（`order`/`订单号`…） |
| CN_USCC P | 0.989 | 0.995 | GB 32100 登记管理部门码 + 机构类别码组合表 |
| CN_BANK_CARD R / F1 | 0.346 / 0.512 | 0.998 / 0.996 | 增加上下文词 `转账`、`账户`、`汇款`、`收款`、`打款` |
| clean 误报 | 1545（IP 1435、卡号 96、USCC 14） | 10（卡号 9、IP 1） | 同上三项 |

改动说明：
1. **去掉 `RegexSet` 预筛**（偏离 spec 第 3 节）。
   - 剖析结果（当时的一次性剖析，没有存原始文件）：11 个内置模式合成一个 `RegexSet` 后，仅 `matches()` 一项在 clean 语料上只有 16–24 MB/s；同样 11 个正则分别跑 `is_match`，合计约 85 MB/s。
   - 现在 `Analyzer` 直接逐个调用识别器，`Recognizer::pattern()` 已删除。
2. **负向上下文** `PatternRecognizer::negative_context`（`src/recognizers/context.rs`）：命中前紧挨着某个完整的字段名时扣 0.5 分。只看紧挨着的那个单词，所以 `server 10.0.0.1` 不会被 `ver` 误伤，有单元测试覆盖。
3. **USCC 前两位组合表**（`src/recognizers/cn_uscc.rs`）。
4. **银行卡上下文词**。
5. **流式还原** `StreamRestorer`（`src/stream.rs`）：功能项，不影响上表数字。proptest 验证任意切分下的输出都等于整段 `restore` 的结果。

**风险【事实 + 推断】**：第 2、4 项用到的词，是看了这份语料的误报和漏报之后加的。这些模板和 hard negative 出自同一个生成器，所以 after 组的准确率**很可能高估**了在真实流量上的表现。要消除这个偏差，需要一份独立来源的标注语料，本轮没有。第 1、3 项不依赖语料（性能改动；GB 32100 标准规则）。

## 6. 复现

```bash
bench/run.sh after
```

脚本会依次：
- 构建 `bench/rust` 下的 runner；
- `uv sync` Python 环境；
- 安装 redacter 0.23.0，拉取 pii-vault v0.2.0 的规则；
- 用固定种子生成语料，按顺序跑全部工具；
- 汇总到 `bench/results/<标签>/`。

只跑部分工具：

```bash
TOOLS="piiguard pii-veil" bench/run.sh quick
```

只重新汇总、不重跑：

```bash
bench/report.sh after
```

原始文件：`bench/results/<标签>/raw/` 下有各工具的预测（`*.pred.jsonl.gz`，字符偏移）、逐篇计时（`*.timing.json.gz`，纳秒）、`/usr/bin/time -l` 输出（`*.time.txt`）和加载时间（`*.load.json`）。
