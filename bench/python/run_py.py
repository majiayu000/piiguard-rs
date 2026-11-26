"""Python 竞品 runner，输出格式与 bench/rust 的 runner 相同。

用法：uv run python run_py.py <tool> <out_dir> <corpus.jsonl>...
tools:
  presidio-spacy  AnalyzerEngine() 默认配置（spaCy en_core_web_lg，含 SpacyRecognizer NER）
  presidio-rules  NoOpNlpEngine + 去掉 SpacyRecognizer：只跑规则识别器（无 NER、无词形上下文增强）
  scrubadub       scrubadub.Scrubber() 默认检测器
"""

import json
import sys
import time
from pathlib import Path

WARMUP = 50


def load(name: str):
    if name == "presidio-spacy":
        from presidio_analyzer import AnalyzerEngine

        engine = AnalyzerEngine()
        return lambda t: [(r.start, r.end, r.entity_type) for r in engine.analyze(t, language="en")]
    if name == "presidio-rules":
        from presidio_analyzer import AnalyzerEngine, RecognizerRegistry
        from presidio_analyzer.nlp_engine import NoOpNlpEngine

        registry = RecognizerRegistry()
        registry.load_predefined_recognizers(languages=["en"])
        registry.remove_recognizer("SpacyRecognizer")
        engine = AnalyzerEngine(registry=registry, nlp_engine=NoOpNlpEngine(models=[{"lang_code": "en", "model_name": "none"}]), supported_languages=["en"])
        return lambda t: [(r.start, r.end, r.entity_type) for r in engine.analyze(t, language="en")]
    if name == "scrubadub":
        import scrubadub

        scrubber = scrubadub.Scrubber()
        return lambda t: [(f.beg, f.end, f.type) for f in scrubber.iter_filth(t)]
    raise SystemExit(f"unknown tool {name}")


def run_corpus(detect, name: str, corpus: Path, out_dir: Path) -> None:
    docs = [json.loads(line) for line in corpus.open(encoding="utf-8")]
    for d in docs[:WARMUP]:
        detect(d["text"])
    ns, preds = [], []
    for d in docs:
        t0 = time.perf_counter_ns()
        spans = detect(d["text"])
        ns.append(time.perf_counter_ns() - t0)
        preds.append({"id": d["id"], "spans": [{"start": s, "end": e, "label": lb} for s, e, lb in spans]})
    stem = corpus.stem
    with (out_dir / f"{name}.{stem}.pred.jsonl").open("w", encoding="utf-8") as fh:
        for p in preds:
            fh.write(json.dumps(p, ensure_ascii=False) + "\n")
    total_bytes = sum(len(d["text"].encode("utf-8")) for d in docs)
    timing = {"tool": name, "corpus": stem, "docs": len(docs), "bytes": total_bytes, "ns": ns}
    (out_dir / f"{name}.{stem}.timing.json").write_text(json.dumps(timing))


def main() -> None:
    name, out_dir = sys.argv[1], Path(sys.argv[2])
    out_dir.mkdir(parents=True, exist_ok=True)
    t0 = time.perf_counter()
    detect = load(name)
    load_ms = (time.perf_counter() - t0) * 1e3
    for c in sys.argv[3:]:
        run_corpus(detect, name, Path(c), out_dir)
    (out_dir / f"{name}.load.json").write_text(json.dumps({"load_ms": load_ms}))


if __name__ == "__main__":
    main()
