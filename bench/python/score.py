"""汇总基准：按实体的 precision/recall/F1、吞吐、延迟、内存峰值、无 PII 语料误报数。

用法：uv run python score.py <raw_dir> <corpus_dir> <summary.json>
匹配规则：预测与金标区间有重叠且类型兼容（见 labels.compatible）即 TP，每个金标/预测最多匹配一次。
"""

import json
import re
import statistics
import sys
from collections import Counter
from pathlib import Path

from labels import EVAL_LABELS, GOLD, RAW, SUPPORTED, compatible


def load_jsonl(path: Path) -> dict:
    return {d["id"]: d for d in map(json.loads, path.open(encoding="utf-8"))}


def max_rss_mb(raw: Path, tool: str) -> float | None:
    sizes = []
    for f in raw.glob(f"{tool}*.time.txt"):
        sizes += [int(m) for m in re.findall(r"(\d+)\s+maximum resident set size", f.read_text())]
    return round(max(sizes) / 2**20, 1) if sizes else None


def perf(raw: Path, tool: str, corpus: str) -> dict:
    t = json.loads((raw / f"{tool}.{corpus}.timing.json").read_text())
    mb = t["bytes"] / 1e6
    if t["ns"]:
        ns = sorted(t["ns"])
        total = sum(ns) / 1e9
        q = statistics.quantiles(ns, n=100, method="inclusive")
        return {"mb_s": round(mb / total, 2), "p50_us": round(q[49] / 1e3, 1), "p99_us": round(q[98] / 1e3, 1),
                "panics": t.get("panics", 0), "timed": "per-doc"}
    return {"mb_s": round(mb / (t["wall_ns"] / 1e9), 2), "p50_us": None, "p99_us": None,
            "panics": 0, "timed": "process-wall"}


def accuracy(tool: str, gold: dict, pred: dict) -> dict:
    tp_g, n_g, tp_p, n_p, other = Counter(), Counter(), Counter(), Counter(), Counter()
    for doc_id, g in gold.items():
        gs = [(s["start"], s["end"], GOLD.get(s["label"], s["label"])) for s in g["spans"]]
        ps = [(s["start"], s["end"], RAW.get(s["label"])) for s in pred[doc_id]["spans"]]
        used = [False] * len(ps)
        for gs_, ge, gl in gs:
            n_g[gl] += 1
            for i, (ps_, pe, pl) in enumerate(ps):
                if not used[i] and pl and ps_ < ge and gs_ < pe and compatible(gl, pl):
                    used[i] = True
                    tp_g[gl] += 1
                    break
        for i, (_, _, pl) in enumerate(ps):
            if pl is None:
                other[pred[doc_id]["spans"][i]["label"]] += 1
                continue
            n_p[pl] += 1
            tp_p[pl] += used[i]
    rows, sup = {}, SUPPORTED.get(tool, set())
    for lb in EVAL_LABELS:
        if lb not in sup:
            rows[lb] = None
            continue
        p = tp_p[lb] / n_p[lb] if n_p[lb] else None
        r = tp_g[lb] / n_g[lb] if n_g[lb] else None
        f = 2 * p * r / (p + r) if p and r else 0.0
        rows[lb] = {"p": p, "r": r, "f1": f, "gold": n_g[lb], "pred": n_p[lb]}
    tg = sum(tp_g[lb] for lb in sup)
    ng = sum(n_g[lb] for lb in sup)
    tpp = sum(tp_p[lb] for lb in sup)
    npp = sum(n_p[lb] for lb in sup)
    mp, mr = (tpp / npp if npp else 0.0), (tg / ng if ng else 0.0)
    micro = {"p": mp, "r": mr, "f1": 2 * mp * mr / (mp + mr) if mp + mr else 0.0}
    return {"per_entity": rows, "micro_supported": micro, "other_type_preds": dict(other.most_common(8))}


def false_positives(pred: dict) -> dict:
    labels = Counter(s["label"] for d in pred.values() for s in d["spans"])
    eval_fp = sum(n for lb, n in labels.items() if lb in RAW)
    return {"all_types": sum(labels.values()), "eval_types": eval_fp, "top": dict(labels.most_common(6))}


def main() -> None:
    raw, corpus, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    gold, clean_docs = load_jsonl(corpus / "labeled.jsonl"), load_jsonl(corpus / "clean.jsonl")
    tools = sorted(p.name.split(".")[0] for p in raw.glob("*.labeled.timing.json"))
    summary = {"corpus": {"labeled_docs": len(gold), "clean_docs": len(clean_docs)}, "tools": {}}
    for tool in tools:
        load = raw / f"{tool}.load.json"
        summary["tools"][tool] = {
            "load_ms": json.loads(load.read_text())["load_ms"] if load.exists() else None,
            "max_rss_mb": max_rss_mb(raw, tool),
            "perf_labeled": perf(raw, tool, "labeled"),
            "perf_clean": perf(raw, tool, "clean"),
            "accuracy": accuracy(tool, gold, load_jsonl(raw / f"{tool}.labeled.pred.jsonl")),
            "clean_fp": false_positives(load_jsonl(raw / f"{tool}.clean.pred.jsonl")),
        }
    out.write_text(json.dumps(summary, ensure_ascii=False, indent=2))
    print_tables(summary)


def fmt(x, nd=3):
    return "—" if x is None else (f"{x:.{nd}f}" if isinstance(x, float) else str(x))


def print_tables(summary: dict) -> None:
    tools = summary["tools"]
    print("| 工具 | 吞吐 labeled MB/s | 吞吐 clean MB/s | p50 µs | p99 µs | 内存峰值 MB | 加载 ms | panic 篇数 |")
    print("|---|---|---|---|---|---|---|---|")
    for t, v in tools.items():
        a, b = v["perf_labeled"], v["perf_clean"]
        print(f"| {t} | {a['mb_s']} | {b['mb_s']} | {fmt(a['p50_us'])} | {fmt(a['p99_us'])} | "
              f"{fmt(v['max_rss_mb'])} | {fmt(v['load_ms'], 0)} | {a['panics'] + b['panics']} |")
    print("\n| 工具 | " + " | ".join(EVAL_LABELS) + " | micro P / R / F1 |")
    print("|---|" + "---|" * (len(EVAL_LABELS) + 1))
    for t, v in tools.items():
        cells = []
        for lb in EVAL_LABELS:
            r = v["accuracy"]["per_entity"][lb]
            cells.append("不支持" if r is None else f"{fmt(r['p'], 2)} / {fmt(r['r'], 2)} / {fmt(r['f1'], 2)}")
        m = v["accuracy"]["micro_supported"]
        print(f"| {t} | " + " | ".join(cells) + f" | {m['p']:.3f} / {m['r']:.3f} / {m['f1']:.3f} |")
    print("\n| 工具 | 误报（全部类型） | 误报（评测类型） | 主要标签 |")
    print("|---|---|---|---|")
    for t, v in tools.items():
        fp = v["clean_fp"]
        top = ", ".join(f"{k} {n}" for k, n in fp["top"].items())
        print(f"| {t} | {fp['all_types']} | {fp['eval_types']} | {top} |")


if __name__ == "__main__":
    main()
