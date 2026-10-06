#!/usr/bin/env python3
"""
Phase 10 - parse all synth_duration result files into a single CSV.

Reads every results/cluster_*/synth_duration-*.txt file, extracts the
[RUN_METRICS] block, and writes one row per run to
TCC/artifacts/phase10/analysis/synth_duration_raw.csv

Run from repository root:
  python TCC/artifacts/phase10/analysis/parse_synth_duration.py
"""

import re
from pathlib import Path
import pandas as pd

RESULTS_DIR = Path("results")
OUT_DIR = Path("TCC/artifacts/phase10/analysis")
OUT_DIR.mkdir(parents=True, exist_ok=True)

TAG_RE = re.compile(
    r"synth_duration-f(?P<f_pct>\d+)pct_(?P<n>\d+)w_m5xl_(?P<strategy>replace|degraded|adaptive)"
)
KV_RE = re.compile(r"^(\w+)=(.*)$")

rows = []
for f in sorted(RESULTS_DIR.glob("cluster_*/synth_duration-*.txt")):
    text = f.read_text()
    if "[RUN_METRICS]" not in text or "status=SUCCESS" not in text:
        continue
    block = text.split("[RUN_METRICS]")[1].split("[/RUN_METRICS]")[0]
    kv = {}
    for line in block.strip().splitlines():
        m = KV_RE.match(line.strip())
        if m:
            kv[m.group(1)] = m.group(2)

    tag_m = TAG_RE.search(kv.get("task_tag", ""))
    if not tag_m:
        continue

    row = {
        "file": f.name,
        "f_pct": int(tag_m.group("f_pct")),
        "n_workers": int(tag_m.group("n")),
        "strategy": tag_m.group("strategy"),
        "ft_wall_time_s": float(kv.get("ft_wall_time_s", "nan")),
        "phase1_s": float(kv.get("phase1_s", "nan")),
        "phase2a_s": float(kv.get("phase2a_s", "nan")),
        "phase2b_s": float(kv.get("phase2b_s", "nan")),
        "phase2c_s": float(kv.get("phase2c_s", "nan")),
        "phase3_s": float(kv.get("phase3_s", "nan")),
        "auto_failure_trigger_secs": float(kv.get("auto_failure_trigger_secs", "nan")),
    }
    # decision event, for the adaptive run only
    if tag_m.group("strategy") == "adaptive":
        row["decision"] = "DEGRADED_RESUME" if "degraded_resume" in text else (
            "REPLACE_RESUME" if "recovery_completed" in text and "degraded_resume" not in text else "UNKNOWN"
        )
    rows.append(row)

df = pd.DataFrame(rows)
df = df.sort_values(["n_workers", "f_pct", "strategy", "file"]).reset_index(drop=True)
df.to_csv(OUT_DIR / "synth_duration_raw.csv", index=False)
print(f"{len(df)} rows written to {OUT_DIR / 'synth_duration_raw.csv'}")
print(df.groupby(["n_workers", "f_pct", "strategy"]).size())
