"""各工具原始标签 → 评测用统一标签，以及各工具声明支持的统一标签（用于判定召回是否适用）。

支持范围来源：各工具源码中的识别器列表（见 docs/comparison.md 的出处列）。
"""

EVAL_LABELS = ["EMAIL", "CREDIT_CARD", "IBAN", "IP_ADDRESS", "US_SSN", "PHONE", "CN_ID_CARD", "CN_USCC", "CN_BANK_CARD"]

# 金标标签 → 评测标签（中美电话合并为 PHONE，因为多数工具只有通用电话类型）
GOLD = {"US_PHONE": "PHONE", "CN_PHONE": "PHONE"}

RAW_TO_EVAL = {
    "EMAIL": ["EMAIL", "EMAIL_ADDRESS", "email", "Email"],
    "CREDIT_CARD": ["CREDIT_CARD", "credit_card", "payment-card"],
    "IBAN": ["IBAN", "IBAN_CODE", "iban"],
    "IP_ADDRESS": ["IP_ADDRESS", "IPV4", "IPV6", "IP_V4", "IP_V6", "ipv4", "ipv6", "IpAddress"],
    # pii-veil 的 NATIONAL_ID 是多国证件号的合并类型（源码中含 US SSN，不含中国身份证），按 SSN 计分
    "US_SSN": ["US_SSN", "SSN", "social_security_number", "us-ssn", 'Custom("SSN")', "NATIONAL_ID"],
    "PHONE": ["US_PHONE", "CN_PHONE", "PHONE_NUMBER", "PHONE_US", "PHONE", "phone", "PhoneNumber",
              "phone-national", "phone-international"],
    "CN_ID_CARD": ["CN_ID_CARD", "CN_RIC", "chinese-resident-id"],
    "CN_USCC": ["CN_USCC"],
    "CN_BANK_CARD": ["CN_BANK_CARD"],
}
RAW = {raw: ev for ev, raws in RAW_TO_EVAL.items() for raw in raws}

# 银行卡本身就是支付卡：CREDIT_CARD 与 CN_BANK_CARD 互相可匹配
COMPATIBLE = {"CREDIT_CARD": {"CREDIT_CARD", "CN_BANK_CARD"}, "CN_BANK_CARD": {"CN_BANK_CARD", "CREDIT_CARD"}}

ALL = set(EVAL_LABELS)
COMMON = {"EMAIL", "CREDIT_CARD", "IBAN", "IP_ADDRESS", "US_SSN", "PHONE"}
SUPPORTED = {
    "piiguard": ALL,
    "piiguard-v0.1": ALL,
    "presidio-spacy": COMMON,
    "presidio-rules": COMMON,
    "presidio-rs": COMMON | {"CN_ID_CARD"},
    # pii-vault 规则目录含 cn_bank_card / cn_id_card / cn_phone，但无统一社会信用代码
    "pii-vault": ALL - {"CN_USCC"},
    "pii-veil": COMMON,
    "cloakpipe": {"EMAIL", "IP_ADDRESS", "US_SSN", "PHONE"},
    "llm-pii-redact": COMMON,
    "redacter": COMMON | {"CN_ID_CARD"},
    "scrubadub": {"EMAIL", "CREDIT_CARD", "US_SSN", "PHONE"},
}


def compatible(gold: str, pred: str) -> bool:
    return pred in COMPATIBLE.get(gold, {gold})
