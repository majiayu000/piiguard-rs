#!/usr/bin/env bash
# 一条命令复现全部基准：bench/run.sh [结果标签]
# 结果写到 bench/results/<标签>/（summary.md、summary.json、env.txt、raw/ 原始预测与计时）。
# 可用 TOOLS="piiguard presidio-rs" 只跑部分工具。
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
LABEL=${1:-current}
WORK=$ROOT/target/bench
RAW=$WORK/raw-$LABEL
RES=$ROOT/bench/results/$LABEL
RUNNER=$ROOT/bench/rust/target/release/piiguard-bench-runners
REDACTER=$ROOT/target/tools/bin/redacter
RUST_TOOLS="piiguard presidio-rs pii-vault pii-veil cloakpipe llm-pii-redact"
PY_TOOLS="presidio-rules presidio-spacy scrubadub"
TOOLS=${TOOLS:-"$RUST_TOOLS $PY_TOOLS redacter"}

cd "$ROOT"
cargo build --release --manifest-path bench/rust/Cargo.toml
(cd bench/python && uv sync --python 3.12 --quiet)
[ -x "$REDACTER" ] || cargo install redacter --version 0.23.0 --locked --root "$ROOT/target/tools"
[ -d "$WORK/vendor/pii-vault" ] || git clone --quiet --depth 1 --branch v0.2.0 \
  https://github.com/Jiansen/pii-vault "$WORK/vendor/pii-vault"
export PII_VAULT_SPEC=$WORK/vendor/pii-vault/spec/recognizers

# 无 PII 代码语料：bench/rust/Cargo.lock 锁定版本的 serde_json 与 aho-corasick 源码
CODE_DIRS=$(cargo metadata --manifest-path bench/rust/Cargo.toml --format-version 1 | python3 -c '
import json, sys, os
for p in json.load(sys.stdin)["packages"]:
    if p["name"] in ("serde_json", "aho-corasick"):
        print(os.path.join(os.path.dirname(p["manifest_path"]), "src"))')
(cd bench/python && uv run python gen_corpus.py "$WORK/corpus" $CODE_DIRS)
CORPORA="$WORK/corpus/labeled.jsonl $WORK/corpus/clean.jsonl"

rm -rf "$RAW" && mkdir -p "$RAW"
for t in $TOOLS; do
  echo "== $t"
  if [[ " $RUST_TOOLS " == *" $t "* ]]; then
    /usr/bin/time -l "$RUNNER" "$t" "$RAW" $CORPORA 2> "$RAW/$t.time.txt"
  elif [[ " $PY_TOOLS " == *" $t "* ]]; then
    (cd bench/python && /usr/bin/time -l .venv/bin/python -W ignore run_py.py "$t" "$RAW" $CORPORA 2> "$RAW/$t.time.txt")
  else
    (cd bench/python && uv run python run_redacter.py "$REDACTER" "$RAW" $CORPORA)
  fi
done

"$ROOT/bench/report.sh" "$LABEL"
