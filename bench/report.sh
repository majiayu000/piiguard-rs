#!/usr/bin/env bash
# 汇总 target/bench/raw-<标签> 的原始结果，写入 bench/results/<标签>/。run.sh 末尾会调用。
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
LABEL=${1:-current}
WORK=$ROOT/target/bench
RAW=$WORK/raw-$LABEL
RES=$ROOT/bench/results/$LABEL
REDACTER=$ROOT/target/tools/bin/redacter
cd "$ROOT"

mkdir -p "$RES/raw"
{
  echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "machine: $(sysctl -n hw.model 2>/dev/null) / $(sysctl -n machdep.cpu.brand_string 2>/dev/null) / $(sysctl -n hw.ncpu 2>/dev/null) cores / $(( $(sysctl -n hw.memsize 2>/dev/null) / 1073741824 )) GB"
  echo "os: $(sw_vers -productName 2>/dev/null) $(sw_vers -productVersion 2>/dev/null)"
  echo "rustc: $(rustc --version)"
  echo "python: $(bench/python/.venv/bin/python --version)"
  echo "piiguard git: $(git rev-parse --short HEAD)$(git diff --quiet HEAD -- src Cargo.toml || echo ' + uncommitted changes')"
  echo "rust crates:"
  cargo metadata --manifest-path bench/rust/Cargo.toml --format-version 1 | python3 -c '
import json, sys
want = {"presidio-analyzer", "pii-vault", "pii-veil-core", "cloakpipe-core", "llm-pii-redact", "regex"}
for p in json.load(sys.stdin)["packages"]:
    if p["name"] in want:
        print("  %s %s" % (p["name"], p["version"]))'
  echo "  redacter: $("$REDACTER" --help | head -1)"
  echo "python packages:"
  (cd bench/python && uv pip list 2>/dev/null | grep -Ei '^(presidio|scrubadub|spacy |en-core|phonenumbers)' | sed 's/^/  /')
  echo "presidio-spacy: AnalyzerEngine() default = SpacyNlpEngine(en_core_web_lg) + default en recognizers, threshold 0"
  echo "presidio-rules: NoOpNlpEngine + default en recognizers minus SpacyRecognizer, threshold 0"
  echo "cloakpipe: DetectionConfig defaults + phone_numbers=true + ip_addresses=true"
  echo "timing: per-doc wall time of the detect call only, after 50-doc warmup, single thread; redacter = whole-process wall time"
  echo "memory: /usr/bin/time -l maximum resident set size of the runner process (includes loading the corpus)"
} > "$RES/env.txt"
(cd bench/python && .venv/bin/python score.py "$RAW" "$WORK/corpus" "$RES/summary.json") > "$RES/summary.md"
for f in "$RAW"/*.pred.jsonl "$RAW"/*.timing.json; do gzip -c "$f" > "$RES/raw/$(basename "$f").gz"; done
cp "$RAW"/*.time.txt "$RAW"/*.load.json "$RES/raw/" 2>/dev/null || true
cat "$RES/summary.md"
