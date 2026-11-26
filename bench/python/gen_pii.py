"""合成 PII 生成器：全部按算法生成，不对应任何已知真实个人或机构。

- 邮箱只用 RFC 2606 保留域名（example.com/org/net）
- IP 只用 RFC 5737 / RFC 3849 文档地址段与私有地址段
- 美国电话只用 555-0100..0199（NANP 保留给虚构用途的号段）
"""

import random

USCC_CHARSET = "0123456789ABCDEFGHJKLMNPQRTUWXY"
ID_WEIGHTS = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2]
CN_REGIONS = ["110101", "310104", "440305", "330106", "510107", "420102", "320102", "610113"]
CN_SEGMENTS = ["130", "135", "138", "139", "150", "152", "158", "166", "177", "181", "186", "189", "199"]
FIRST = ["alice", "bob", "carol", "dave", "erin", "frank", "grace", "heidi", "wang.lei", "li.na"]
DOMAINS = ["example.com", "example.org", "example.net", "mail.example.com"]


def luhn_complete(prefix: str) -> str:
    total = 0
    for i, c in enumerate(reversed(prefix)):
        d = int(c)
        if i % 2 == 0:
            d *= 2
            if d > 9:
                d -= 9
        total += d
    return prefix + str((10 - total % 10) % 10)


def digits(rng: random.Random, n: int) -> str:
    return "".join(rng.choice("0123456789") for _ in range(n))


def email(rng: random.Random) -> str:
    return f"{rng.choice(FIRST)}{rng.randint(1, 999)}@{rng.choice(DOMAINS)}"


def credit_card(rng: random.Random) -> str:
    kind = rng.choice(["visa", "mc", "amex", "discover"])
    if kind == "visa":
        raw = luhn_complete("4" + digits(rng, 14))
    elif kind == "mc":
        raw = luhn_complete(str(rng.randint(51, 55)) + digits(rng, 13))
    elif kind == "amex":
        raw = luhn_complete(rng.choice(["34", "37"]) + digits(rng, 12))
        return raw if rng.random() < 0.5 else f"{raw[:4]} {raw[4:10]} {raw[10:]}"
    else:
        raw = luhn_complete("6011" + digits(rng, 11))
    sep = rng.choice(["", " ", "-"])
    return sep.join(raw[i:i + 4] for i in range(0, len(raw), 4))


def cn_bank_card(rng: random.Random) -> str:
    raw = luhn_complete("62" + digits(rng, rng.choice([14, 15, 16])))
    return raw if rng.random() < 0.7 else " ".join(raw[i:i + 4] for i in range(0, len(raw), 4))


IBAN_BBAN = {
    "DE": [("n", 18)], "GB": [("a", 4), ("n", 14)], "FR": [("n", 23)],
    "NL": [("a", 4), ("n", 10)], "ES": [("n", 20)], "IT": [("a", 1), ("n", 22)],
}


def iban(rng: random.Random) -> str:
    cc = rng.choice(list(IBAN_BBAN))
    bban = ""
    for kind, n in IBAN_BBAN[cc]:
        alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ" if kind == "a" else "0123456789"
        bban += "".join(rng.choice(alphabet) for _ in range(n))
    num = "".join(str(int(c, 36)) for c in bban + cc + "00")
    check = 98 - int(num) % 97
    raw = f"{cc}{check:02d}{bban}"
    return raw if rng.random() < 0.5 else " ".join(raw[i:i + 4] for i in range(0, len(raw), 4))


def ip(rng: random.Random) -> str:
    if rng.random() < 0.75:
        net = rng.choice(["192.0.2", "198.51.100", "203.0.113", "10.20.30", "192.168.1"])
        return f"{net}.{rng.randint(1, 254)}"
    groups = [f"{rng.randint(0, 0xffff):x}" for _ in range(4)]
    return "2001:db8:" + ":".join(groups) + f"::{rng.randint(1, 0xfff):x}"


def us_ssn(rng: random.Random) -> str:
    area = rng.choice([a for a in range(1, 900) if a != 666])
    raw = f"{area:03d}{rng.randint(1, 99):02d}{rng.randint(1, 9999):04d}"
    return rng.choice([f"{raw[:3]}-{raw[3:5]}-{raw[5:]}", raw])


def us_phone(rng: random.Random) -> str:
    area = rng.choice(["212", "415", "312", "617", "206", "303"])
    line = f"01{rng.randint(0, 99):02d}"
    return rng.choice([f"({area}) 555-{line}", f"{area}-555-{line}", f"+1 {area} 555 {line}", f"{area}555{line}"])


def cn_phone(rng: random.Random) -> str:
    raw = rng.choice(CN_SEGMENTS) + digits(rng, 8)
    return rng.choice([raw, f"{raw[:3]} {raw[3:7]} {raw[7:]}", f"+86 {raw}", f"{raw[:3]}-{raw[3:7]}-{raw[7:]}"])


def cn_id(rng: random.Random) -> str:
    y, m = rng.randint(1950, 2005), rng.randint(1, 12)
    d = rng.randint(1, 28)
    body = f"{rng.choice(CN_REGIONS)}{y}{m:02d}{d:02d}{digits(rng, 3)}"
    s = sum(int(c) * w for c, w in zip(body, ID_WEIGHTS))
    return body + "10X98765432"[s % 11]


def cn_uscc(rng: random.Random) -> str:
    body = rng.choice(["91", "92", "93", "12", "52"]) + rng.choice(CN_REGIONS)
    body += "".join(rng.choice(USCC_CHARSET) for _ in range(9))
    w = [pow(3, i, 31) for i in range(17)]
    s = sum(USCC_CHARSET.index(c) * wi for c, wi in zip(body, w))
    return body + USCC_CHARSET[(31 - s % 31) % 31]


GENERATORS = {
    "EMAIL": email, "CREDIT_CARD": credit_card, "IBAN": iban, "IP_ADDRESS": ip,
    "US_SSN": us_ssn, "US_PHONE": us_phone, "CN_PHONE": cn_phone, "CN_ID_CARD": cn_id,
    "CN_USCC": cn_uscc, "CN_BANK_CARD": cn_bank_card,
}


def corrupt(rng: random.Random, value: str) -> str:
    """把最后一位数字改掉，得到校验位错误的"假 PII"（硬负样本）。"""
    i = max(j for j, c in enumerate(value) if c.isdigit())
    return value[:i] + str((int(value[i]) + rng.randint(1, 9)) % 10) + value[i + 1:]
