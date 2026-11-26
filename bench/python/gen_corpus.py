"""生成带标注的合成语料 corpus/labeled.jsonl 与无 PII 语料 corpus/clean.jsonl。

用法：uv run python gen_corpus.py <out_dir> <code_dir>...   （种子固定为 SEED）
标注为字符偏移；硬负样本（订单号、时间戳、版本号、校验位错误的号码）不标注。
"""

import json
import random
import sys
from pathlib import Path

from gen_pii import GENERATORS, corrupt, digits

SEED = 20261126
N_LABELED = 4000
N_LOG_DOCS = 1500
CODE_CHUNK_LINES = 40

TEMPLATES_EN = [
    "Please contact {EMAIL} for the invoice.",
    "Card on file: {CREDIT_CARD}, expires 09/29.",
    "Wire the refund to IBAN {IBAN} by Friday.",
    "Login from {IP_ADDRESS} at {TS}.",
    "My SSN is {US_SSN}.",
    "Form received with {US_SSN} attached.",
    "Call me at {US_PHONE} after 5pm.",
    "Order {ORDER} shipped, tracking {ORDER}.",
    "Release {VER} built at {TS}, request id {REQID}.",
    "The card number {BAD_CREDIT_CARD} was rejected by the bank.",
    "Customer {EMAIL} paid with {CREDIT_CARD} from {IP_ADDRESS}.",
    "Account {BAD_IBAN} failed validation.",
    "Reach the on-call at {US_PHONE} or {EMAIL}.",
]
TEMPLATES_ZH = [
    "我的身份证号是{CN_ID_CARD}，手机{CN_PHONE}。",
    "请把款打到银行卡{CN_BANK_CARD}，户名见附件。",
    "转账至 {CN_BANK_CARD} 即可。",
    "公司统一社会信用代码：{CN_USCC}。",
    "联系电话 {CN_PHONE}，邮箱 {EMAIL}。",
    "{CN_PHONE} 来电咨询订单 {ORDER}。",
    "订单号 {ORDER}，下单时间 {TS}。",
    "身份证 {BAD_CN_ID_CARD} 校验失败。",
    "供应商 {CN_USCC} 的对公账户 {CN_BANK_CARD}。",
    "服务器 {IP_ADDRESS} 在 {TS} 告警，版本 {VER}。",
    "信用代码 {BAD_CN_USCC} 不存在。",
    "卡号 {BAD_CN_BANK_CARD} 无效，请核对。",
]


def negative(rng: random.Random, kind: str) -> str:
    if kind == "ORDER":
        return digits(rng, 1).replace("0", "2") + digits(rng, rng.randint(13, 19))
    if kind == "TS":
        return rng.choice([
            f"2026-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}T{rng.randint(0, 23):02d}:{rng.randint(0, 59):02d}:{rng.randint(0, 59):02d}Z",
            str(1_764_000_000_000 + rng.randint(0, 10**9)),
            str(1_764_000_000 + rng.randint(0, 10**6)),
        ])
    if kind == "VER":
        return rng.choice([f"v{rng.randint(0, 9)}.{rng.randint(0, 20)}.{rng.randint(0, 99)}",
                           f"{rng.randint(1, 9)}.{rng.randint(0, 20)}.{rng.randint(0, 9)}.{rng.randint(10, 999)}"])
    if kind == "REQID":
        return "%032x" % rng.getrandbits(128)
    raise ValueError(kind)


def fill(rng: random.Random, template: str, spans: list, offset: int) -> str:
    out, i = "", 0
    while (j := template.find("{", i)) != -1:
        k = template.index("}", j)
        out += template[i:j]
        slot = template[j + 1:k]
        if slot in GENERATORS:
            value = GENERATORS[slot](rng)
            spans.append({"start": offset + len(out), "end": offset + len(out) + len(value), "label": slot})
        elif slot.startswith("BAD_"):
            value = corrupt(rng, GENERATORS[slot[4:]](rng))
        else:
            value = negative(rng, slot)
        out += value
        i = k + 1
    return out + template[i:]


def labeled(rng: random.Random):
    for doc_id in range(N_LABELED):
        text, spans = "", []
        for _ in range(rng.randint(1, 4)):
            zh = rng.random() < 0.5
            sep = "" if zh or not text else " "
            text += sep
            text += fill(rng, rng.choice(TEMPLATES_ZH if zh else TEMPLATES_EN), spans, len(text))
        yield {"id": doc_id, "text": text, "spans": spans}


def log_docs(rng: random.Random):
    for doc_id in range(N_LOG_DOCS):
        lines = []
        for _ in range(rng.randint(3, 12)):
            lines.append(
                f"{negative(rng, 'TS')} INFO req={negative(rng, 'REQID')} order={negative(rng, 'ORDER')} "
                f"latency_ms={rng.randint(1, 5000)} tokens={rng.randint(1, 8192)} model=gpt-4o-2024-08-06 "
                f"ver={negative(rng, 'VER')} path=/v1/chat/completions?stream=true status=200"
            )
        yield {"id": f"log-{doc_id}", "text": "\n".join(lines), "spans": []}


def code_docs(dirs: list[str]):
    files = sorted(p for d in dirs for p in Path(d).rglob("*.rs"))
    for f in files:
        lines = f.read_text(encoding="utf-8").splitlines()
        for n in range(0, len(lines), CODE_CHUNK_LINES):
            chunk = "\n".join(lines[n:n + CODE_CHUNK_LINES])
            if chunk.strip():
                yield {"id": f"code-{f.parent.parent.name}/{f.name}:{n + 1}", "text": chunk, "spans": []}


def write(path: Path, docs) -> int:
    with path.open("w", encoding="utf-8") as fh:
        count = 0
        for d in docs:
            fh.write(json.dumps(d, ensure_ascii=False) + "\n")
            count += 1
    return count


def main() -> None:
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    rng = random.Random(SEED)
    n1 = write(out / "labeled.jsonl", labeled(rng))
    clean = list(log_docs(rng)) + list(code_docs(sys.argv[2:]))
    n2 = write(out / "clean.jsonl", clean)
    print(f"labeled={n1} clean={n2} seed={SEED}")


if __name__ == "__main__":
    main()
