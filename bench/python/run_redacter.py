"""redacter（CLI，local-rules 检测器）runner。

redacter 只有命令行、按文件扫描，所以每篇文档写成一个文件，对整个目录跑一次
`redacter scan -d local-rules`，从 JSON 报告取出偏移。计时是整个进程的墙钟时间
（含进程启动和文件 I/O），只用于粗略吞吐，不给逐篇延迟。

用法：uv run python run_redacter.py <redacter_bin> <out_dir> <corpus.jsonl>...
"""

import json
import shutil
import subprocess
import sys
import time
from pathlib import Path


def to_char_span(text: str, start: int, end: int, value: str) -> tuple[int, int]:
    """报告里的偏移可能是字节偏移；用 value 校验后换成字符偏移。"""
    if text[start:end] == value:
        return start, end
    raw = text.encode("utf-8")
    if raw[start:end].decode("utf-8", errors="replace") == value:
        return len(raw[:start].decode("utf-8")), len(raw[:end].decode("utf-8"))
    i = text.find(value)
    return (i, i + len(value)) if i >= 0 else (-1, -1)


def run_corpus(binary: str, corpus: Path, out_dir: Path) -> None:
    docs = [json.loads(line) for line in corpus.open(encoding="utf-8")]
    work = out_dir / f"redacter-docs-{corpus.stem}"
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir(parents=True)
    for i, d in enumerate(docs):
        (work / f"{i}.txt").write_text(d["text"], encoding="utf-8")
    report = out_dir / f"redacter.{corpus.stem}.report.json"
    t0 = time.perf_counter_ns()
    proc = subprocess.run(
        ["/usr/bin/time", "-l", binary, "scan", "-d", "local-rules", "--show-values", "--report-file", str(report), str(work)],
        stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, check=False)
    wall = time.perf_counter_ns() - t0
    (out_dir / f"redacter.{corpus.stem}.time.txt").write_text(proc.stderr)
    found = {}
    for f in json.loads(report.read_text())["files"]:
        idx = int(Path(f["path"]).stem)
        text = docs[idx]["text"]
        spans = []
        for x in f.get("findings") or []:
            loc = x.get("location") or {}
            if "start" not in loc:
                continue
            s, e = to_char_span(text, loc["start"], loc["end"], x.get("value") or "")
            if s >= 0:
                spans.append({"start": s, "end": e, "label": x["label"]})
        found[idx] = spans
    with (out_dir / f"redacter.{corpus.stem}.pred.jsonl").open("w", encoding="utf-8") as fh:
        for i, d in enumerate(docs):
            fh.write(json.dumps({"id": d["id"], "spans": found.get(i, [])}, ensure_ascii=False) + "\n")
    total = sum(len(d["text"].encode("utf-8")) for d in docs)
    timing = {"tool": "redacter", "corpus": corpus.stem, "docs": len(docs), "bytes": total, "ns": [], "wall_ns": wall}
    (out_dir / f"redacter.{corpus.stem}.timing.json").write_text(json.dumps(timing))
    shutil.rmtree(work)


def main() -> None:
    binary, out_dir = sys.argv[1], Path(sys.argv[2])
    out_dir.mkdir(parents=True, exist_ok=True)
    for c in sys.argv[3:]:
        run_corpus(binary, Path(c), out_dir)


if __name__ == "__main__":
    main()
