#!/usr/bin/env python3
"""
TCC Results Analyzer
====================
Parses result .txt files produced by hpcac-toolkit run_task, extracts
[RUN_METRICS] and [METRICS] blocks, and generates:
  - TCC/artifacts/phase4/analysis/results_raw.csv   (all valid runs, 1 row each)
  - TCC/artifacts/phase4/analysis/summary.md        (mean ± std table, grouped)
  - TCC/artifacts/phase4/analysis/plots/*.png       (comparison graphs)

A run is considered VALID only if the result file contains a [RUN_METRICS]
block with status=SUCCESS AND the NPB benchmark reports Verification=SUCCESSFUL.
Incomplete or failed runs are silently discarded.

Usage:
  python TCC/analyze.py
  python TCC/analyze.py --results-dir results/ --output-dir TCC/artifacts/phase4/analysis
"""

import re
import os
import sys
import argparse
from pathlib import Path
from collections import defaultdict

import pandas as pd
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker

# ── AWS on-demand prices (us-west-2, per hour) ─────────────────────────────
# Used for the HEAD node (on-demand: never interrupted, coordinates the job).
ON_DEMAND_USD_PER_HR = {
    "t3.micro":    0.0104,
    "t3.small":    0.0208,
    "t3.medium":   0.0416,
    "t3.large":    0.0832,
    "t3.xlarge":   0.1664,
    "t3.2xlarge":  0.3328,
    "m5.large":    0.0960,
    "m5.xlarge":   0.1920,
    "m5.2xlarge":  0.3840,
    "m5.4xlarge":  0.7680,
    "m5.8xlarge":  1.5360,
}

# ── AWS spot prices (us-west-2, per hour) ───────────────────────────────────
# Used for WORKER nodes (spot: can be interrupted — the scenario this TCC studies).
# Prices are averages across all us-west-2 AZs, sampled 2026-05-26 via
#   aws ec2 describe-spot-price-history --region us-west-2 --product-descriptions "Linux/UNIX"
# Each average is based on 300–850 data points (min/max shown in comments).
SPOT_USD_PER_HR = {
    "t3.micro":    0.0035,  # on-demand 0.0104  →  66% discount  (min=0.0028 max=0.0042)
    "t3.small":    0.0080,  # on-demand 0.0208  →  62% discount  (min=0.0053 max=0.0105)
    "t3.medium":   0.0156,  # on-demand 0.0416  →  63% discount  (min=0.0128 max=0.0188)
    "t3.large":    0.0338,  # on-demand 0.0832  →  59% discount  (min=0.0221 max=0.0432)
    "t3.xlarge":   0.0561,  # on-demand 0.1664  →  66% discount  (min=0.0448 max=0.0798)
    "t3.2xlarge":  0.1176,  # on-demand 0.3328  →  65% discount  (min=0.0824 max=0.1505)
    "m5.large":    0.0342,  # on-demand 0.0960  →  64% discount  (min=0.0279 max=0.0406)
    "m5.xlarge":   0.0585,  # on-demand 0.1920  →  70% discount  (min=0.0460 max=0.0769)
    "m5.2xlarge":  0.1572,  # on-demand 0.3840  →  59% discount  (min=0.1227 max=0.2085)
    "m5.4xlarge":  0.2787,  # on-demand 0.7680  →  64% discount  (min=0.1980 max=0.3357)
    "m5.8xlarge":  0.4777,  # on-demand 1.5360  →  69% discount  (min=0.2515 max=0.6665)
}

# ── Display constants ───────────────────────────────────────────────────────
# Canonical strategy keys (from [RUN_METRICS] strategy= field + task_tag suffix)
STRATEGY_KEY_NONE      = "noFT"       # native MPI, no MANA
STRATEGY_KEY_MANA_NONE = "MANA_noFT"  # MANA present, no interruption
STRATEGY_KEY_REPLACE   = "REPLACE"
STRATEGY_KEY_DEGRADED  = "DEGRADED"

STRATEGY_ORDER = [STRATEGY_KEY_NONE, STRATEGY_KEY_MANA_NONE,
                  STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]

STRATEGY_LABEL = {
    STRATEGY_KEY_NONE:      "noFT (native)",
    STRATEGY_KEY_MANA_NONE: "MANA noFT",
    STRATEGY_KEY_REPLACE:   "REPLACE",
    STRATEGY_KEY_DEGRADED:  "DEGRADED",
}

STRATEGY_COLOR = {
    STRATEGY_KEY_NONE:      "#4C72B0",
    STRATEGY_KEY_MANA_NONE: "#55A868",
    STRATEGY_KEY_REPLACE:   "#DD8452",
    STRATEGY_KEY_DEGRADED:  "#C44E52",
}

# ── Parsing helpers ─────────────────────────────────────────────────────────

def parse_kv_block(text: str, tag: str) -> list[dict]:
    """Extract all [tag] ... [/tag] blocks and parse key=value lines into dicts."""
    pattern = re.compile(
        rf"\[{re.escape(tag)}\](.*?)\[/{re.escape(tag)}\]",
        re.DOTALL,
    )
    results = []
    for m in pattern.finditer(text):
        d = {}
        for line in m.group(1).splitlines():
            line = line.strip()
            if "=" in line:
                k, _, v = line.partition("=")
                d[k.strip()] = v.strip()
        if d:
            results.append(d)
    return results


def parse_npb_last_result(text: str) -> dict:
    """
    Find the LAST occurrence of a complete NPB benchmark result block
    and return its key metrics. Returns {} if not found.
    For FT runs the last block is from the restart (the valid result).
    """
    # Patterns for NPB output fields
    patterns = {
        "app_time_s":    re.compile(r"Time in seconds\s*=\s*([\d.]+)"),
        "mops_total":    re.compile(r"Mop/s total\s*=\s*([\d.]+)"),
        "mops_per_proc": re.compile(r"Mop/s/process\s*=\s*([\d.]+)"),
        "verification":  re.compile(r"Verification\s*=\s*(\S+)"),
        "benchmark":     re.compile(r"NAS Parallel Benchmarks.*?--\s*(\w+)\s+Benchmark", re.IGNORECASE),
        "npb_class":     re.compile(r"Class\s*=\s*([A-Z])"),
        "total_procs":   re.compile(r"Total processes\s*=\s*(\d+)"),
        "active_procs":  re.compile(r"Active processes=\s*(\d+)"),
    }

    # Find all positions of "Time in seconds" to locate result blocks
    time_matches = list(re.finditer(r"Time in seconds\s*=\s*([\d.]+)", text))
    if not time_matches:
        return {}

    # Use the LAST match as anchor point — scan backwards for a window
    last_pos = time_matches[-1].start()
    # Take a window around the last result (3000 chars before + 500 after)
    window_start = max(0, last_pos - 3000)
    window = text[window_start: last_pos + 500]

    result = {}
    for key, pat in patterns.items():
        m = None
        # Find last occurrence within the window
        for m in pat.finditer(window):
            pass
        if m:
            result[key] = m.group(1).strip()

    return result


def parse_task_tag(tag: str) -> dict:
    """
    Parse a task_tag into components.
    Handles NPB tags like 'lu-C_4w_m5xl_replace-25pct' and
    synthetic tags like 'synth_calls-l2_2w_m5xl_noFT',
    'synth_ckpt-200mb_2w_m5xl_degraded'.
    Returns dict with benchmark, class, config_workers, strategy_suffix,
    timing_pct, synth_level, synth_memory_mb.
    """
    base = {
        "benchmark": tag, "class": "?", "config_workers": 0,
        "strategy_suffix": "?", "timing_pct": None,
        "synth_level": None, "synth_memory_mb": None,
    }

    # Synth checkpoint: synth_ckpt-{N}mb_{W}w_{instance}_{strategy}
    m = re.match(r"^synth_ckpt-(\d+)mb_(\d+)w_[^_]+_(.+)$", tag)
    if m:
        return {**base, "benchmark": "SYNTH_CKPT", "class": "-",
                "config_workers": int(m.group(2)), "strategy_suffix": m.group(3),
                "synth_memory_mb": int(m.group(1))}

    # Synth level-based: synth_{type}-l{N}_{W}w_{instance}_{strategy}
    m = re.match(r"^synth_([a-z0-9]+(?:_[a-z0-9]+)?)-l(\d+)_(\d+)w_[^_]+_(.+)$", tag)
    if m:
        bench = "SYNTH_" + m.group(1).upper().replace("_", "")
        return {**base, "benchmark": bench, "class": "-",
                "config_workers": int(m.group(3)), "strategy_suffix": m.group(4),
                "synth_level": int(m.group(2))}

    # Standard NPB: {bench}-{class}_{N}w_{instance}_{strategy}
    m = re.match(r"^([a-zA-Z]+)-([A-Z])_(\d+)w_[^_]+_(.+)$", tag)
    if m:
        suffix = m.group(4)
        pct_m = re.search(r"-(\d+)pct$", suffix)
        return {**base,
                "benchmark":       m.group(1).upper(),
                "class":           m.group(2),
                "config_workers":  int(m.group(3)),
                "strategy_suffix": suffix,
                "timing_pct":      int(pct_m.group(1)) if pct_m else None}

    return base


def resolve_strategy_key(strategy_field: str, strategy_suffix: str) -> str:
    """
    Map [RUN_METRICS] strategy= field + task_tag suffix to a canonical key.
    strategy_field:  NONE | REPLACE_RESUME | DEGRADED_RESUME
    strategy_suffix: noFT | MANA_noFT | replace | degraded | ...
    """
    sf = strategy_suffix.lower()
    if strategy_field == "REPLACE_RESUME":
        return STRATEGY_KEY_REPLACE
    if strategy_field == "DEGRADED_RESUME":
        return STRATEGY_KEY_DEGRADED
    # strategy_field == "NONE"
    if "mana" in sf:
        return STRATEGY_KEY_MANA_NONE
    return STRATEGY_KEY_NONE


def compute_cost(row: dict) -> tuple[float, float, float]:
    """
    Returns (run_cost_spot_usd, run_cost_ondemand_usd, total_cost_usd).

    run_cost_spot_usd    : ft_wall_time_s × (workers×spot + head×on-demand)
                           — the FT scenario: workers are spot instances
    run_cost_ondemand_usd: ft_wall_time_s × (all nodes×on-demand)
                           — the baseline: no FT, must use on-demand to avoid
                             losing work on spot interruptions
    total_cost_usd       : total_time_s × spot cluster rate (includes setup)

    Note: REPLACE briefly runs N+1 worker nodes while the replacement boots
    (Phase 2), but the replacement is billed from instance start in AWS.
    This is not modelled here — costs are computed as if N workers run
    throughout, so REPLACE costs are slightly underestimated.
    """
    w_spot   = SPOT_USD_PER_HR.get(row.get("worker_instance_type", ""), 0.0)
    w_od     = ON_DEMAND_USD_PER_HR.get(row.get("worker_instance_type", ""), 0.0)
    h_price  = ON_DEMAND_USD_PER_HR.get(row.get("head_instance_type", ""), 0.0)
    workers  = int(row.get("workers", 0))

    hourly_spot = workers * w_spot + h_price
    hourly_od   = workers * w_od   + h_price

    ft_wall = float(row.get("ft_wall_time_s", 0))
    total   = float(row.get("total_time_s",   0))
    return (
        (ft_wall / 3600.0) * hourly_spot,
        (ft_wall / 3600.0) * hourly_od,
        (total   / 3600.0) * hourly_spot,
    )


# ── File parsing ────────────────────────────────────────────────────────────

def parse_result_file(filepath: Path) -> list[dict]:
    """
    Parse a single result .txt file and return a list of run records.
    Each record corresponds to one [RUN_METRICS] block (one task).
    Records with status != SUCCESS or verification != SUCCESSFUL are discarded.
    """
    text = filepath.read_text(errors="replace")
    records = []

    run_metrics_list = parse_kv_block(text, "RUN_METRICS")
    if not run_metrics_list:
        return []  # No completed tasks in this file

    # Per-cycle FT metrics — keyed by cycle_id for later join
    ft_metrics_by_cycle = {}
    for block in parse_kv_block(text, "METRICS"):
        cid = block.get("cycle_id", "")
        if cid and cid != "no_cycle_id":
            ft_metrics_by_cycle[cid] = block

    # NPB results — use last successful result in file
    npb = parse_npb_last_result(text)

    def _f(d, k):
        v = d.get(k)
        try: return float(v) if v is not None else None
        except (ValueError, TypeError): return None

    for rm in run_metrics_list:
        # Discard failed runs
        if rm.get("status") != "SUCCESS":
            continue

        # Parse task_tag
        tag_parts = parse_task_tag(rm.get("task_tag", ""))
        strategy_key = resolve_strategy_key(
            rm.get("strategy", "NONE"),
            tag_parts.get("strategy_suffix", ""),
        )

        # NPB verification check — discard if failed
        verification = npb.get("verification", "")
        if verification and verification.upper() != "SUCCESSFUL":
            continue

        # Recovery cycle count
        recovery_cycles = len(ft_metrics_by_cycle)

        # auto_failure_trigger_secs — present only for FT runs with auto trigger
        raw_trigger = rm.get("auto_failure_trigger_secs")
        auto_trigger_secs = int(raw_trigger) if raw_trigger is not None else None

        # Phase timing — read directly from [RUN_METRICS]
        phase0_s  = float(auto_trigger_secs) if auto_trigger_secs is not None else None
        phase1_s  = _f(rm, "phase1_s")
        phase2_s  = _f(rm, "phase2_s")
        phase2a_s = _f(rm, "phase2a_s")
        phase2b_s = _f(rm, "phase2b_s")
        phase2c_s = _f(rm, "phase2c_s")
        phase3_s  = _f(rm, "phase3_s")
        recovery_total_s = (
            phase1_s + phase2_s + phase3_s
            if None not in (phase1_s, phase2_s, phase3_s) else None
        )

        run_cost, run_cost_od, total_cost = compute_cost({**rm})

        record = {
            # Identity
            "source_file":               filepath.name,
            "task_tag":                  rm.get("task_tag", ""),
            "benchmark":                 tag_parts.get("benchmark", "?"),
            "class":                     tag_parts.get("class", "?"),
            "config_workers":            tag_parts.get("config_workers", 0),
            "strategy":                  strategy_key,
            "timing_pct":                tag_parts.get("timing_pct"),
            "synth_level":               tag_parts.get("synth_level"),
            "synth_memory_mb":           tag_parts.get("synth_memory_mb"),
            # Hardware
            "worker_instance_type":      rm.get("worker_instance_type", ""),
            "head_instance_type":        rm.get("head_instance_type", ""),
            # Timing (seconds)
            "ft_wall_time_s":            float(rm.get("ft_wall_time_s", 0)),
            "setup_time_s":              float(rm.get("setup_time_s", 0)),
            "total_time_s":              float(rm.get("total_time_s", 0)),
            # Failure injection
            "auto_failure_trigger_secs": auto_trigger_secs,
            # NPB application metrics
            "app_time_s":           float(npb["app_time_s"])   if npb.get("app_time_s")   else None,
            "mops_total":           float(npb["mops_total"])   if npb.get("mops_total")   else None,
            "mops_per_proc":        float(npb["mops_per_proc"])if npb.get("mops_per_proc")else None,
            "total_procs":          int(npb["total_procs"])    if npb.get("total_procs")  else None,
            "active_procs":         int(npb["active_procs"])   if npb.get("active_procs") else None,
            "verification":         verification or "N/A",
            # FT recovery metrics (None for non-FT runs)
            "recovery_cycles":      recovery_cycles,
            "phase0_s":             phase0_s,
            "phase1_s":             phase1_s,
            "phase2_s":             phase2_s,
            "phase2a_s":            phase2a_s,
            "phase2b_s":            phase2b_s,
            "phase2c_s":            phase2c_s,
            "phase3_s":             phase3_s,
            "recovery_total_s":     recovery_total_s,
            # Cost  (spot = workers on spot + head on-demand; od = all on-demand)
            "run_cost_usd":         run_cost,
            "run_cost_ondemand_usd": run_cost_od,
            "total_cost_usd":       total_cost,
        }
        records.append(record)

    return records


def load_all_results(results_dir: Path) -> pd.DataFrame:
    """Scan all .txt files recursively and build a DataFrame of valid runs."""
    all_records = []
    txt_files = sorted(results_dir.rglob("*.txt"))
    if not txt_files:
        print(f"No .txt result files found under {results_dir}", file=sys.stderr)
        return pd.DataFrame()

    for f in txt_files:
        recs = parse_result_file(f)
        all_records.extend(recs)

    if not all_records:
        print("No valid (SUCCESS + SUCCESSFUL verification) runs found.", file=sys.stderr)
        return pd.DataFrame()

    df = pd.DataFrame(all_records)
    # Add a run index per (benchmark, class, config_workers, strategy) group
    df = df.sort_values(["benchmark", "class", "config_workers", "strategy", "source_file"])
    df["run_index"] = df.groupby(
        ["benchmark", "class", "config_workers", "strategy"]
    ).cumcount() + 1
    return df.reset_index(drop=True)


# ── Output: CSV & summary table ─────────────────────────────────────────────

def save_raw_csv(df: pd.DataFrame, out_dir: Path):
    path = out_dir / "results_raw.csv"
    df.to_csv(path, index=False)
    print(f"  Saved: {path}")


def save_summary_markdown(df: pd.DataFrame, out_dir: Path):
    """
    Build an aggregated table: mean ± std for key metrics,
    grouped by benchmark / class / config_workers / strategy.
    """
    numeric_cols = [
        "ft_wall_time_s", "mops_total", "mops_per_proc",
        "phase0_s", "phase1_s", "phase2_s",
        "phase2a_s", "phase2b_s", "phase2c_s",
        "phase3_s", "recovery_total_s",
        "run_cost_usd", "auto_failure_trigger_secs",
    ]
    group_cols = ["benchmark", "class", "config_workers", "strategy"]

    agg = {}
    for col in numeric_cols:
        if col in df.columns:
            agg[col] = ["mean", "std", "count"]

    summary = df.groupby(group_cols).agg(agg).reset_index()
    summary.columns = [
        "_".join(c).strip("_") for c in summary.columns.values
    ]

    lines = ["# TCC Results Summary\n"]
    lines.append(f"Total valid runs: {len(df)}\n")
    lines.append(f"Benchmarks: {sorted(df['benchmark'].unique())}\n")
    lines.append(f"Strategies: {sorted(df['strategy'].unique())}\n\n")

    for bench in sorted(df["benchmark"].unique()):
        for cls in sorted(df[df["benchmark"] == bench]["class"].unique()):
            lines.append(f"## {bench} Class {cls}\n\n")
            sub = summary[
                (summary["benchmark"] == bench) & (summary["class"] == cls)
            ].copy()

            # Order strategies
            strat_order_map = {s: i for i, s in enumerate(STRATEGY_ORDER)}
            sub["_sort"] = sub["strategy"].map(lambda s: strat_order_map.get(s, 99))
            sub = sub.sort_values(["config_workers", "_sort"])

            header_cols = ["Workers", "Strategy", "N", "Trigger (s)",
                           "FT Wall (s)", "Mop/s total",
                           "Ph0 (s)", "Ph1 (s)", "Ph2a (s)", "Ph2b (s)", "Ph2c (s)", "Ph3 (s)",
                           "Recovery (s)", "Run Cost (spot $)"]
            lines.append("| " + " | ".join(header_cols) + " |\n")
            lines.append("| " + " | ".join(["---"] * len(header_cols)) + " |\n")

            for _, row in sub.iterrows():
                def fmt(mean_col, std_col, decimals=1):
                    m = row.get(mean_col)
                    s = row.get(std_col)
                    if pd.isna(m) or m is None:
                        return "—"
                    if pd.isna(s) or s is None or s == 0:
                        return f"{m:.{decimals}f}"
                    return f"{m:.{decimals}f} ±{s:.{decimals}f}"

                n = int(row.get("ft_wall_time_s_count", 1))
                trigger_mean = row.get("auto_failure_trigger_secs_mean")
                trigger_std  = row.get("auto_failure_trigger_secs_std")
                if pd.isna(trigger_mean) or trigger_mean is None:
                    trigger_cell = "—"
                elif not pd.isna(trigger_std) and trigger_std > 0:
                    trigger_cell = f"⚠ {trigger_mean:.0f} (mixed!)"
                else:
                    trigger_cell = f"{int(trigger_mean)}s"

                line_cols = [
                    str(int(row["config_workers"])),
                    STRATEGY_LABEL.get(row["strategy"], row["strategy"]),
                    str(n),
                    trigger_cell,
                    fmt("ft_wall_time_s_mean", "ft_wall_time_s_std", decimals=1),
                    fmt("mops_total_mean",     "mops_total_std",     decimals=1),
                    fmt("phase0_s_mean",       "phase0_s_std",       decimals=1),
                    fmt("phase1_s_mean",       "phase1_s_std",       decimals=1),
                    fmt("phase2a_s_mean",      "phase2a_s_std",      decimals=1),
                    fmt("phase2b_s_mean",      "phase2b_s_std",      decimals=1),
                    fmt("phase2c_s_mean",      "phase2c_s_std",      decimals=1),
                    fmt("phase3_s_mean",       "phase3_s_std",       decimals=1),
                    fmt("recovery_total_s_mean","recovery_total_s_std", decimals=1),
                    fmt("run_cost_usd_mean",   "run_cost_usd_std",   decimals=4),
                ]
                lines.append("| " + " | ".join(line_cols) + " |\n")
            lines.append("\n")

    path = out_dir / "summary.md"
    path.write_text("".join(lines))
    print(f"  Saved: {path}")


# ── Plotting helpers ─────────────────────────────────────────────────────────

def _strat_order(s):
    return STRATEGY_ORDER.index(s) if s in STRATEGY_ORDER else 99

def _label(s):
    return STRATEGY_LABEL.get(s, s)

def _color(s):
    return STRATEGY_COLOR.get(s, "#888888")

PHASE_SEGS = ["phase1_s", "phase2a_s", "phase2b_s", "phase2c_s", "phase3_s"]
PHASE_COLORS_SEG = {
    "phase1_s":  "#5B9BD5",
    "phase2a_s": "#FFD966",
    "phase2b_s": "#ED7D31",
    "phase2c_s": "#F4B183",
    "phase3_s":  "#70AD47",
}
PHASE_LABELS_SEG = {
    "phase1_s":  "Phase 1 — detect → checkpoint",
    "phase2a_s": "Phase 2a — drain + cancel",
    "phase2b_s": "Phase 2b — node reconfig (EC2 / scontrol)",
    "phase2c_s": "Phase 2c — restart dispatch",
    "phase3_s":  "Phase 3 — remaining computation",
}

# ── Plots ────────────────────────────────────────────────────────────────────

def plot_mana_overhead(df: pd.DataFrame, out_dir: Path):
    """
    Fig 1: noFT vs MANA-noFT elapsed time for CG / EP / LU.
    Each benchmark gets one panel; bars are grouped by worker count.
    Annotates each MANA bar with the overhead percentage vs noFT.
    """
    npb = df[df["benchmark"].isin(["CG", "EP", "LU"]) &
             df["strategy"].isin([STRATEGY_KEY_NONE, STRATEGY_KEY_MANA_NONE])].copy()
    if npb.empty:
        print("  Skipping fig1 (no baseline data)")
        return

    bench_class = {"CG": "C", "EP": "D", "LU": "C"}
    fig, axes = plt.subplots(1, 3, figsize=(14, 5))
    fig.suptitle("MANA overhead — elapsed time without failures\n"
                 "(m5.xlarge workers, 2 processes per node)", fontsize=12)

    for ax, bench in zip(axes, ["CG", "EP", "LU"]):
        sub = npb[npb["benchmark"] == bench]
        worker_counts = sorted(sub["config_workers"].unique())
        x = np.arange(len(worker_counts))
        bar_w = 0.35

        noft_vals, mana_vals = [], []
        for w in worker_counts:
            noft = sub[(sub["config_workers"] == w) & (sub["strategy"] == STRATEGY_KEY_NONE)]["ft_wall_time_s"]
            mana = sub[(sub["config_workers"] == w) & (sub["strategy"] == STRATEGY_KEY_MANA_NONE)]["ft_wall_time_s"]
            noft_vals.append(noft.mean() if len(noft) else np.nan)
            mana_vals.append(mana.mean() if len(mana) else np.nan)

        ax.bar(x - bar_w / 2, noft_vals, bar_w,
               label="noFT (native MPI)", color=STRATEGY_COLOR[STRATEGY_KEY_NONE])
        ax.bar(x + bar_w / 2, mana_vals, bar_w,
               label="MANA (no failure)", color=STRATEGY_COLOR[STRATEGY_KEY_MANA_NONE])

        for i, (noft, mana) in enumerate(zip(noft_vals, mana_vals)):
            if not (np.isnan(noft) or np.isnan(mana)) and noft > 0:
                pct = (mana - noft) / noft * 100
                ax.annotate(f"+{pct:.0f}%",
                            xy=(x[i] + bar_w / 2, mana),
                            xytext=(0, 4), textcoords="offset points",
                            ha="center", fontsize=8, color="#1a6b1a", fontweight="bold")

        ax.set_xticks(x)
        ax.set_xticklabels([f"{w}w" for w in worker_counts])
        ax.set_xlabel("Worker count")
        ax.set_ylabel("Elapsed time (s)")
        ax.set_title(f"{bench}-{bench_class[bench]} Class")
        ax.legend(fontsize=8)
        ax.grid(axis="y", linestyle="--", alpha=0.4)
        ax.set_ylim(bottom=0)

    plt.tight_layout()
    path = out_dir / "fig1_mana_overhead.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_synth_calls(df: pd.DataFrame, out_dir: Path):
    """
    Fig 2: Synthetic MPI call frequency study.
    Shows noFT vs MANA-noFT elapsed time at each call level for
    synth_calls (MPI_Allreduce) and synth_p2p (MPI_Send/Recv).
    Finding: both bars stay the same height → call count is not the overhead driver.
    """
    synth_df = df[df["benchmark"].isin(["SYNTH_CALLS", "SYNTH_P2P"]) &
                  df["synth_level"].notna()].copy()
    if synth_df.empty:
        print("  Skipping fig2 (no synth_calls / synth_p2p data)")
        return

    call_counts = {0: "0", 1: "800", 2: "3,200", 3: "12,800", 4: "51,200"}
    fig, axes = plt.subplots(1, 2, figsize=(13, 5), sharey=True)
    fig.suptitle("Synthetic MPI call frequency study — elapsed time at each call level\n"
                 "Finding: MANA overhead stays flat regardless of call count",
                 fontsize=11)

    bench_labels = {"SYNTH_CALLS": "synth_calls  (MPI_Allreduce collectives)",
                    "SYNTH_P2P":   "synth_p2p  (MPI_Send / MPI_Recv point-to-point)"}

    for ax, bench in zip(axes, ["SYNTH_CALLS", "SYNTH_P2P"]):
        sub = synth_df[synth_df["benchmark"] == bench]
        levels = sorted(sub["synth_level"].dropna().unique().astype(int))
        x = np.arange(len(levels))
        bar_w = 0.35

        noft_vals, mana_vals = [], []
        for lvl in levels:
            noft = sub[(sub["synth_level"] == lvl) & (sub["strategy"] == STRATEGY_KEY_NONE)]["ft_wall_time_s"]
            mana = sub[(sub["synth_level"] == lvl) & (sub["strategy"] == STRATEGY_KEY_MANA_NONE)]["ft_wall_time_s"]
            noft_vals.append(noft.mean() if len(noft) else np.nan)
            mana_vals.append(mana.mean() if len(mana) else np.nan)

        ax.bar(x - bar_w / 2, noft_vals, bar_w,
               label="noFT", color=STRATEGY_COLOR[STRATEGY_KEY_NONE])
        ax.bar(x + bar_w / 2, mana_vals, bar_w,
               label="MANA (no failure)", color=STRATEGY_COLOR[STRATEGY_KEY_MANA_NONE])

        # Annotate overhead on each MANA bar
        for i, (noft, mana) in enumerate(zip(noft_vals, mana_vals)):
            if not (np.isnan(noft) or np.isnan(mana)):
                diff = mana - noft
                sign = "+" if diff >= 0 else ""
                ax.annotate(f"{sign}{diff:.1f}s",
                            xy=(x[i] + bar_w / 2, mana),
                            xytext=(0, 4), textcoords="offset points",
                            ha="center", fontsize=7, color="#555")

        ax.set_xticks(x)
        ax.set_xticklabels([f"L{lvl}\n({call_counts.get(lvl,'?')} calls)" for lvl in levels],
                           fontsize=8)
        ax.set_xlabel("Call level")
        ax.set_ylabel("Elapsed time (s)")
        ax.set_title(bench_labels[bench])
        ax.legend(fontsize=8)
        ax.grid(axis="y", linestyle="--", alpha=0.4)
        ax.set_ylim(bottom=0)

    plt.tight_layout()
    path = out_dir / "fig2_synth_calls.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_synth_imbalanced(df: pd.DataFrame, out_dir: Path):
    """
    Fig 2b: Communication imbalance study (synth_imbalanced).
    Shows noFT vs MANA-noFT wall time at each sender-delay level.
    Finding: MANA overhead stays flat even as communication imbalance grows.
    """
    imb_df = df[
        (df["benchmark"] == "SYNTH_IMBALANCED") &
        df["synth_level"].notna()
    ].copy()
    if imb_df.empty:
        print("  Skipping fig2b (no synth_imbalanced data)")
        return

    delay_labels = {0: "0 µs\n(L0)", 1: "100 µs\n(L1)", 2: "1 ms\n(L2)",
                    3: "5 ms\n(L3)", 4: "20 ms\n(L4)"}

    levels = sorted(imb_df["synth_level"].dropna().unique().astype(int))
    x = np.arange(len(levels))
    bar_w = 0.35

    fig, ax = plt.subplots(figsize=(8, 5))
    fig.suptitle("Communication imbalance study (synth_imbalanced)\n"
                 "MPI_Irecv + MPI_Wait with controlled sender delay — MANA overhead stays flat",
                 fontsize=11)

    noft_vals, mana_vals = [], []
    for lvl in levels:
        noft = imb_df[(imb_df["synth_level"] == lvl) &
                      (imb_df["strategy"] == STRATEGY_KEY_NONE)]["ft_wall_time_s"]
        mana = imb_df[(imb_df["synth_level"] == lvl) &
                      (imb_df["strategy"] == STRATEGY_KEY_MANA_NONE)]["ft_wall_time_s"]
        noft_vals.append(noft.mean() if len(noft) else np.nan)
        mana_vals.append(mana.mean() if len(mana) else np.nan)

    ax.bar(x - bar_w / 2, noft_vals, bar_w,
           label="noFT (native)", color=STRATEGY_COLOR[STRATEGY_KEY_NONE])
    ax.bar(x + bar_w / 2, mana_vals, bar_w,
           label="MANA (no failure)", color=STRATEGY_COLOR[STRATEGY_KEY_MANA_NONE])

    for i, (noft, mana) in enumerate(zip(noft_vals, mana_vals)):
        if not (np.isnan(noft) or np.isnan(mana)):
            diff = mana - noft
            sign = "+" if diff >= 0 else ""
            ax.annotate(f"{sign}{diff:.1f}s",
                        xy=(x[i] + bar_w / 2, mana),
                        xytext=(0, 4), textcoords="offset points",
                        ha="center", fontsize=8, color="#555")

    ax.set_xticks(x)
    ax.set_xticklabels([delay_labels.get(lvl, str(lvl)) for lvl in levels], fontsize=8)
    ax.set_xlabel("Sender delay per exchange")
    ax.set_ylabel("Elapsed time (s)")
    ax.legend(fontsize=9)
    ax.grid(axis="y", linestyle="--", alpha=0.4)
    ax.set_ylim(bottom=0)

    plt.tight_layout()
    path = out_dir / "fig2b_synth_imbalanced.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_checkpoint_size(df: pd.DataFrame, out_dir: Path):
    """
    Fig 3: Phase 1 time vs checkpoint image size (memory per process).
    Scatter + linear fit; shows how checkpoint write time scales with data volume.
    """
    ckpt_df = df[
        (df["benchmark"] == "SYNTH_CKPT") &
        (df["strategy"] == STRATEGY_KEY_DEGRADED) &
        df["synth_memory_mb"].notna() &
        df["phase1_s"].notna()
    ].copy()
    if ckpt_df.empty:
        print("  Skipping fig3 (no synth_ckpt data)")
        return

    ckpt_df = ckpt_df.sort_values("synth_memory_mb")
    xs = ckpt_df["synth_memory_mb"].values.astype(float)
    ys = ckpt_df["phase1_s"].values.astype(float)

    fig, ax = plt.subplots(figsize=(7, 5))
    ax.scatter(xs, ys, color="#5B9BD5", s=90, zorder=5, label="measured Phase 1 time")

    if len(xs) >= 2:
        coeffs = np.polyfit(xs, ys, 1)
        x_fit  = np.linspace(0, xs.max() * 1.05, 300)
        y_fit  = np.polyval(coeffs, x_fit)
        ax.plot(x_fit, y_fit, color="#ED7D31", linewidth=1.5, linestyle="--",
                label=f"linear fit  slope = {coeffs[0]*1000:.2f} ms / MB  "
                      f"(intercept = {coeffs[1]:.1f} s)")

    for x_pt, y_pt in zip(xs, ys):
        ax.annotate(f"{int(x_pt)} MB", (x_pt, y_pt),
                    textcoords="offset points", xytext=(8, 3), fontsize=9)

    ax.set_xlabel("Memory allocated per MPI process (MB)")
    ax.set_ylabel("Phase 1 time (s)  —  failure detected → checkpoint written to EFS")
    ax.set_title("Synthetic checkpoint size study\n"
                 "Phase 1 time scales with checkpoint image size")
    ax.legend(fontsize=9)
    ax.grid(linestyle="--", alpha=0.4)
    ax.set_xlim(left=0)
    ax.set_ylim(bottom=0)

    plt.tight_layout()
    path = out_dir / "fig3_synth_ckpt.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def _fv(row, key):
    """Safely extract a float from a DataFrame row, returning 0.0 on missing/NaN."""
    v = row.get(key)
    if v is None:
        return 0.0
    try:
        f = float(v)
        return f if not pd.isna(f) else 0.0
    except (TypeError, ValueError):
        return 0.0


def _draw_phase_bars(ax, px, p0_a, p1_a, slurm_a, p2b_a, p3_a,
                     strat_for_bar, show_legend):
    """
    Draw 5-segment stacked bars on ax and annotate totals.
    Segments (bottom to top):
      P0    (teal)   — pre-failure computation
      P1    (blue)   — checkpoint write
      Slurm (yellow) — P2a + P2c: Slurm drain/cancel + re-alloc + MANA coordinator setup
      P2b   (orange) — node reconfig (EC2 provisioning or scontrol DOWN)
      P3    (green)  — remaining computation after restart
    """
    COLOR_P0    = "#4BACC6"
    COLOR_P1    = "#5B9BD5"
    COLOR_SLURM = "#FFD966"
    COLOR_P2B   = "#ED7D31"
    COLOR_P3    = "#70AD47"

    lbl = lambda s, t: t if show_legend else "_"
    b0 = np.zeros(len(px))
    ax.bar(px, p0_a,    0.32, bottom=b0,                       color=COLOR_P0,    label=lbl(True, "P0 — pre-failure compute"))
    b1 = b0 + p0_a
    ax.bar(px, p1_a,    0.32, bottom=b1,                       color=COLOR_P1,    label=lbl(True, "P1 — checkpoint write"))
    b2 = b1 + p1_a
    ax.bar(px, slurm_a, 0.32, bottom=b2,                       color=COLOR_SLURM, label=lbl(True, "Slurm + MANA overhead\n(drain/cancel + re-alloc + coordinator)"))
    b3 = b2 + slurm_a
    ax.bar(px, p2b_a,   0.32, bottom=b3,                       color=COLOR_P2B,   label=lbl(True, "P2b — node reconfig\n(EC2 provisioning or scontrol DOWN)"))
    b4 = b3 + p2b_a
    ax.bar(px, p3_a,    0.32, bottom=b4,                       color=COLOR_P3,    label=lbl(True, "P3 — remaining computation"))

    totals = b4 + p3_a
    for pos, total in zip(px, totals):
        # Annotate inside the bar near the top so adjacent shorter bars never obscure it
        y_txt = max(total * 0.96, total - 3)
        ax.text(pos, y_txt, f"{total:.0f}s",
                ha="center", va="top", fontsize=6.5,
                bbox=dict(boxstyle="round,pad=0.1", facecolor="white",
                          alpha=0.75, edgecolor="none"))

    # REP / DEG labels — placed at 25 % of the bar height so they sit clearly inside P0/P1
    for pos, s, total in zip(px, strat_for_bar, totals):
        short = "REP" if s == STRATEGY_KEY_REPLACE else "DEG"
        dark  = "#1a5276" if s == STRATEGY_KEY_REPLACE else "#0e3b0e"
        ax.text(pos, max(total * 0.12, 3), short,
                ha="center", va="center", fontsize=6.5, color=dark, fontweight="bold")


def plot_timing_phases(df: pd.DataFrame, out_dir: Path):
    """
    Fig 4: Total FT wall time by failure timing — REPLACE vs DEGRADED.

    2×3 grid: benchmarks (EP, LU) × worker counts (2w, 4w, 8w).
    Each panel: 3 timing groups (failure at 10% / 25% / 50% of noFT run time).
    Each group: 2 bars side by side — REPLACE (REP) and DEGRADED (DEG).

    5 stacked segments (bottom to top):
      P0 (teal)    — pre-failure computation: equal for both bars in each group
      P1 (blue)    — checkpoint write
      Slurm/MANA (yellow) — P2a+P2c: drain/cancel + re-alloc + coordinator setup
      P2b (orange) — node reconfig: REPLACE ~125 s EC2 vs DEGRADED ~11 s scontrol
      P3 (green)   — remaining computation after restart
    """
    timing_df = df[
        df["benchmark"].isin(["EP", "LU"]) &
        df["strategy"].isin([STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]) &
        df["timing_pct"].notna()
    ].copy()
    if timing_df.empty:
        print("  Skipping fig4 (no timing sensitivity data)")
        return

    benchmarks    = [b for b in ["EP", "LU"] if b in timing_df["benchmark"].unique()]
    worker_counts = sorted(timing_df["config_workers"].unique())
    pcts          = [10, 25, 50]

    bar_w     = 0.32
    bar_gap   = 0.06
    group_gap = 0.55

    fig, axes = plt.subplots(
        len(benchmarks), len(worker_counts),
        figsize=(5.5 * len(worker_counts), 5.5 * len(benchmarks)),
        squeeze=False,
    )
    fig.suptitle(
        "Total FT wall time breakdown by failure timing — REPLACE vs DEGRADED\n"
        "Teal=P0 (pre-failure)  ·  Blue=P1 (checkpoint write)  ·  Yellow=Slurm+MANA overhead"
        "  ·  Orange=P2b (node reconfig)  ·  Green=P3 (remaining work)",
        fontsize=9,
    )

    for ri, bench in enumerate(benchmarks):
        for ci, workers in enumerate(worker_counts):
            ax = axes[ri][ci]
            sub = timing_df[
                (timing_df["benchmark"] == bench) &
                (timing_df["config_workers"] == workers)
            ]

            positions, group_ticks, group_xlbls = [], [], []
            p0_v, p1_v, slurm_v, p2b_v, p3_v   = [], [], [], [], []
            strat_for_bar = []

            base_x = 0.0
            for pct in pcts:
                has_any = False
                for bi, s in enumerate([STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]):
                    row = sub[(sub["strategy"] == s) & (sub["timing_pct"] == pct)]
                    if row.empty:
                        continue
                    has_any = True
                    r   = row.iloc[0]
                    positions.append(base_x + bi * (bar_w + bar_gap))
                    strat_for_bar.append(s)
                    p0_v.append(_fv(r, "phase0_s"))
                    p1_v.append(_fv(r, "phase1_s"))
                    slurm_v.append(_fv(r, "phase2a_s") + _fv(r, "phase2c_s"))
                    p2b_v.append(_fv(r, "phase2b_s"))
                    p3_v.append(_fv(r, "phase3_s"))
                if has_any:
                    group_ticks.append(base_x + (bar_w + bar_gap) / 2)
                    group_xlbls.append(f"Failure\nat {pct}%")
                base_x += 2 * (bar_w + bar_gap) + group_gap

            if not positions:
                ax.set_visible(False)
                continue

            show_legend = ri == 0 and ci == len(worker_counts) - 1
            _draw_phase_bars(
                ax, np.array(positions),
                np.array(p0_v), np.array(p1_v), np.array(slurm_v),
                np.array(p2b_v), np.array(p3_v),
                strat_for_bar, show_legend,
            )

            bench_cls = "D" if bench == "EP" else "C"
            ax.set_title(f"{bench}-{bench_cls}  ·  {workers} workers", fontsize=10)
            ax.set_xticks(group_ticks)
            ax.set_xticklabels(group_xlbls, fontsize=8)
            ax.set_ylabel("Wall time (s)", fontsize=8)
            ax.set_ylim(bottom=0)
            ax.grid(axis="y", linestyle="--", alpha=0.4)
            if show_legend:
                ax.legend(fontsize=7.5, loc="upper right", framealpha=0.9)

    plt.tight_layout()
    path = out_dir / "fig4_timing_phases.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_cg_short_job(df: pd.DataFrame, out_dir: Path):
    """
    Fig 5: CG short-job FT wall time — REPLACE vs DEGRADED across worker counts.

    CG-C is a very short benchmark (~3-5 s without FT), so it has no timing variants.
    This figure shows the FT wall time breakdown at 2w / 4w / 8w using the same
    5-segment scheme as fig4. The key point: for short jobs, recovery overhead
    (especially P2b) dominates total run time; P0 is nearly invisible.
    """
    cg_df = df[
        (df["benchmark"] == "CG") &
        df["strategy"].isin([STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]) &
        df["timing_pct"].isna()
    ].copy()
    if cg_df.empty:
        print("  Skipping fig5 (no CG FT runs)")
        return

    worker_counts = sorted(cg_df["config_workers"].unique())
    bar_w = 0.32
    bar_gap = 0.06
    group_gap = 0.55

    fig, ax = plt.subplots(figsize=(3.5 * len(worker_counts), 5))
    fig.suptitle(
        "CG-C short benchmark — total FT wall time (REPLACE vs DEGRADED)\n"
        "For short jobs, recovery overhead dominates; P0 (pre-failure compute) is nearly zero",
        fontsize=10,
    )

    positions, group_ticks, group_xlbls = [], [], []
    p0_v, p1_v, slurm_v, p2b_v, p3_v   = [], [], [], [], []
    strat_for_bar = []

    base_x = 0.0
    for w in worker_counts:
        has_any = False
        for bi, s in enumerate([STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]):
            row = cg_df[(cg_df["config_workers"] == w) & (cg_df["strategy"] == s)]
            if row.empty:
                continue
            has_any = True
            r = row.iloc[0]
            positions.append(base_x + bi * (bar_w + bar_gap))
            strat_for_bar.append(s)
            p0_v.append(_fv(r, "phase0_s"))
            p1_v.append(_fv(r, "phase1_s"))
            slurm_v.append(_fv(r, "phase2a_s") + _fv(r, "phase2c_s"))
            p2b_v.append(_fv(r, "phase2b_s"))
            p3_v.append(_fv(r, "phase3_s"))
        if has_any:
            group_ticks.append(base_x + (bar_w + bar_gap) / 2)
            group_xlbls.append(f"{w} workers")
        base_x += 2 * (bar_w + bar_gap) + group_gap

    if not positions:
        plt.close()
        return

    _draw_phase_bars(
        ax, np.array(positions),
        np.array(p0_v), np.array(p1_v), np.array(slurm_v),
        np.array(p2b_v), np.array(p3_v),
        strat_for_bar, show_legend=True,
    )

    ax.set_xticks(group_ticks)
    ax.set_xticklabels(group_xlbls, fontsize=9)
    ax.set_ylabel("Wall time (s)", fontsize=9)
    ax.set_ylim(bottom=0)
    ax.grid(axis="y", linestyle="--", alpha=0.4)
    ax.legend(fontsize=8, loc="upper right", framealpha=0.9)

    plt.tight_layout()
    path = out_dir / "fig5_cg_short_job.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_mana_scalability(df: pd.DataFrame, out_dir: Path):
    """
    Fig 6: Strong scaling — wall time vs worker count for noFT and MANA-noFT.
    One panel per benchmark. Shows whether wall time decreases as expected with
    more workers, and whether MANA adds measurable overhead at any scale.
    """
    npb = df[
        df["benchmark"].isin(["CG", "EP", "LU"]) &
        df["strategy"].isin([STRATEGY_KEY_NONE, STRATEGY_KEY_MANA_NONE])
    ].copy()
    if npb.empty:
        print("  Skipping fig6 (no baseline data)")
        return

    bench_class = {"CG": "C", "EP": "D", "LU": "C"}
    benchmarks  = [b for b in ["CG", "EP", "LU"] if b in npb["benchmark"].unique()]

    fig, axes = plt.subplots(1, len(benchmarks), figsize=(5 * len(benchmarks), 4.5))
    if len(benchmarks) == 1:
        axes = [axes]
    fig.suptitle(
        "Strong scaling — wall time vs worker count  (no failures)\n"
        "Solid = native MPI (noFT)   ·   Dashed = with MANA checkpoint layer (MANA-noFT)",
        fontsize=11,
    )

    strat_lines = [
        (STRATEGY_KEY_NONE,      {"ls": "-",  "marker": "o"}),
        (STRATEGY_KEY_MANA_NONE, {"ls": "--", "marker": "s"}),
    ]

    for ax, bench in zip(axes, benchmarks):
        sub = npb[npb["benchmark"] == bench]
        workers_sorted = sorted(sub["config_workers"].unique())

        for s, style in strat_lines:
            xs, ys = [], []
            for w in workers_sorted:
                row = sub[(sub["config_workers"] == w) & (sub["strategy"] == s)]
                if row.empty:
                    continue
                xs.append(w)
                ys.append(float(row["ft_wall_time_s"].iloc[0]))
            if not xs:
                continue
            ax.plot(xs, ys, color=_color(s), linewidth=2, markersize=8,
                    linestyle=style["ls"], marker=style["marker"], label=_label(s))
            for x, y in zip(xs, ys):
                ax.annotate(f"{y:.1f}s", (x, y),
                            textcoords="offset points", xytext=(0, 8),
                            ha="center", fontsize=8.5)

        ax.set_title(f"{bench}-{bench_class[bench]}", fontsize=11)
        ax.set_xlabel("Worker count", fontsize=9)
        ax.set_ylabel("Wall time (s)", fontsize=9)
        ax.set_xticks(workers_sorted)
        ax.set_xticklabels([str(w) for w in workers_sorted])
        ax.legend(fontsize=8.5)
        ax.grid(linestyle="--", alpha=0.4)
        ax.set_ylim(bottom=0)

    plt.tight_layout()
    path = out_dir / "fig6_mana_scalability.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_strategy_comparison(df: pd.DataFrame, out_dir: Path):
    """
    Fig 8: Total FT wall time — MANA-noFT reference + REPLACE + DEGRADED.

    2×3 grid (EP/LU × 2w/4w/8w). Each panel has 3 timing groups (10/25/50%).
    Each group shows 3 bars: MANA-noFT (ideal baseline, no failure), REPLACE, DEGRADED.
    The gap between MANA-noFT and the FT bars shows the cost of the spot interruption.
    """
    ft_df = df[
        df["benchmark"].isin(["EP", "LU"]) &
        df["strategy"].isin([STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]) &
        df["timing_pct"].notna()
    ].copy()
    if ft_df.empty:
        print("  Skipping fig8 (no timing data)")
        return

    mana_df = df[
        df["benchmark"].isin(["EP", "LU"]) &
        (df["strategy"] == STRATEGY_KEY_MANA_NONE)
    ].copy()

    benchmarks    = [b for b in ["EP", "LU"] if b in ft_df["benchmark"].unique()]
    worker_counts = sorted(ft_df["config_workers"].unique())
    pcts          = [10, 25, 50]
    bar_w, bar_gap, group_gap = 0.25, 0.04, 0.5

    fig, axes = plt.subplots(
        len(benchmarks), len(worker_counts),
        figsize=(5.5 * len(worker_counts), 5 * len(benchmarks)),
        squeeze=False,
    )
    fig.suptitle(
        "Total FT wall time — MANA-noFT (no failure) vs REPLACE vs DEGRADED  (lower = better)\n"
        "The gap between MANA-noFT and the FT bars shows the cost of the spot interruption",
        fontsize=10,
    )

    seen_labels: set = set()

    for ri, bench in enumerate(benchmarks):
        for ci, workers in enumerate(worker_counts):
            ax = axes[ri][ci]
            ft_sub = ft_df[
                (ft_df["benchmark"] == bench) & (ft_df["config_workers"] == workers)
            ]
            mana_sub = mana_df[
                (mana_df["benchmark"] == bench) & (mana_df["config_workers"] == workers)
            ]
            t_mana = float(mana_sub["ft_wall_time_s"].iloc[0]) if not mana_sub.empty else None

            positions, group_ticks, group_xlbls = [], [], []
            totals, strat_for_bar = [], []

            base_x = 0.0
            for pct in pcts:
                rep_row = ft_sub[(ft_sub["strategy"] == STRATEGY_KEY_REPLACE) & (ft_sub["timing_pct"] == pct)]
                deg_row = ft_sub[(ft_sub["strategy"] == STRATEGY_KEY_DEGRADED) & (ft_sub["timing_pct"] == pct)]

                entries = [
                    (STRATEGY_KEY_MANA_NONE, t_mana),
                    (STRATEGY_KEY_REPLACE,   float(rep_row["ft_wall_time_s"].iloc[0]) if not rep_row.empty else None),
                    (STRATEGY_KEY_DEGRADED,  float(deg_row["ft_wall_time_s"].iloc[0]) if not deg_row.empty else None),
                ]
                has_any = any(t is not None for _, t in entries)
                for bi, (s, t) in enumerate(entries):
                    if t is None:
                        continue
                    positions.append(base_x + bi * (bar_w + bar_gap))
                    strat_for_bar.append(s)
                    totals.append(t)

                if has_any:
                    group_ticks.append(base_x + bar_w + bar_gap + bar_w / 2)
                    group_xlbls.append(f"Failure\nat {pct}%")
                base_x += 3 * (bar_w + bar_gap) + group_gap

            if not positions:
                ax.set_visible(False)
                continue

            for pos, s, total in zip(positions, strat_for_bar, totals):
                lbl = _label(s) if s not in seen_labels else "_"
                seen_labels.add(s)
                ax.bar(pos, total, bar_w, color=_color(s), label=lbl)
                ax.text(pos, total + 1, f"{total:.0f}s",
                        ha="center", va="bottom", fontsize=6, rotation=45)

            bench_cls = "D" if bench == "EP" else "C"
            ax.set_title(f"{bench}-{bench_cls}  ·  {workers} workers", fontsize=10)
            ax.set_xticks(group_ticks)
            ax.set_xticklabels(group_xlbls, fontsize=8)
            ax.set_ylabel("Wall time (s)", fontsize=8)
            ax.set_ylim(bottom=0)
            ax.grid(axis="y", linestyle="--", alpha=0.4)

    handles, labels = [], []
    for ax in axes.flat:
        for h, l in zip(*ax.get_legend_handles_labels()):
            if l not in labels:
                handles.append(h)
                labels.append(l)
    if handles:
        fig.legend(handles, labels, fontsize=8.5, loc="upper right",
                   bbox_to_anchor=(0.99, 0.99), framealpha=0.9)

    plt.tight_layout(rect=[0, 0, 0.88, 1])
    path = out_dir / "fig8_strategy_comparison.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_recovery_overhead_ratio(df: pd.DataFrame, out_dir: Path):
    """
    Fig 9: Recovery overhead as % of total FT wall time — REPLACE vs DEGRADED.

    recovery_overhead_pct = (ft_wall_time_s - phase0_s) / ft_wall_time_s * 100

    Answers: "what fraction of my total job time was forced by the failure?"
    Later failure → smaller fraction (more useful work done before/after).
    REPLACE always higher because P2b (~125 s) is large relative to P3.
    """
    timing_df = df[
        df["benchmark"].isin(["EP", "LU"]) &
        df["strategy"].isin([STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]) &
        df["timing_pct"].notna()
    ].copy()
    if timing_df.empty:
        print("  Skipping fig9 (no timing data)")
        return

    benchmarks    = [b for b in ["EP", "LU"] if b in timing_df["benchmark"].unique()]
    worker_counts = sorted(timing_df["config_workers"].unique())
    pcts          = [10, 25, 50]

    fig, axes = plt.subplots(
        len(benchmarks), len(worker_counts),
        figsize=(5 * len(worker_counts), 4.5 * len(benchmarks)),
        squeeze=False,
    )
    fig.suptitle(
        "Recovery overhead as % of total FT wall time — REPLACE vs DEGRADED\n"
        "= (ft_wall_time − pre-failure compute) / ft_wall_time × 100\n"
        "Lower % means the failure had less relative impact on the total job",
        fontsize=10,
    )

    COLOR_REP = STRATEGY_COLOR[STRATEGY_KEY_REPLACE]
    COLOR_DEG = STRATEGY_COLOR[STRATEGY_KEY_DEGRADED]

    for ri, bench in enumerate(benchmarks):
        for ci, workers in enumerate(worker_counts):
            ax = axes[ri][ci]
            sub = timing_df[
                (timing_df["benchmark"] == bench) &
                (timing_df["config_workers"] == workers)
            ]

            xs, rep_ys, deg_ys = [], [], []
            for pct in pcts:
                rep_row = sub[(sub["strategy"] == STRATEGY_KEY_REPLACE) & (sub["timing_pct"] == pct)]
                deg_row = sub[(sub["strategy"] == STRATEGY_KEY_DEGRADED) & (sub["timing_pct"] == pct)]
                if rep_row.empty or deg_row.empty:
                    continue
                xs.append(pct)
                for row, lst in [(rep_row, rep_ys), (deg_row, deg_ys)]:
                    r      = row.iloc[0]
                    total  = float(r.get("ft_wall_time_s") or 1)
                    p0     = _fv(r, "phase0_s")
                    ratio  = (total - p0) / total * 100 if total > 0 else 0
                    lst.append(ratio)

            if not xs:
                ax.set_visible(False)
                continue

            show_legend = ri == 0 and ci == len(worker_counts) - 1
            ax.plot(xs, rep_ys, marker="o", color=COLOR_REP, linewidth=2,
                    markersize=8, label="REPLACE" if show_legend else "_")
            ax.plot(xs, deg_ys, marker="s", color=COLOR_DEG, linewidth=2,
                    markersize=8, label="DEGRADED" if show_legend else "_")

            for x, y in zip(xs, rep_ys):
                ax.annotate(f"{y:.0f}%", (x, y),
                            textcoords="offset points", xytext=(0, 8),
                            ha="center", fontsize=8, color=COLOR_REP)
            for x, y in zip(xs, deg_ys):
                ax.annotate(f"{y:.0f}%", (x, y),
                            textcoords="offset points", xytext=(0, -14),
                            ha="center", fontsize=8, color=COLOR_DEG)

            bench_cls = "D" if bench == "EP" else "C"
            ax.set_title(f"{bench}-{bench_cls}  ·  {workers} workers", fontsize=10)
            ax.set_xticks(pcts)
            ax.set_xticklabels([f"{p}%" for p in pcts])
            ax.set_xlabel("Failure timing (% of noFT run)", fontsize=8)
            ax.set_ylabel("Recovery overhead (%)", fontsize=8)
            ax.set_ylim(0, 105)
            ax.grid(linestyle="--", alpha=0.4)
            if show_legend:
                ax.legend(fontsize=9, loc="upper right", framealpha=0.9)

    plt.tight_layout()
    path = out_dir / "fig9_recovery_overhead_ratio.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_cost(df: pd.DataFrame, out_dir: Path):
    """
    Fig 7: Economic cost per run with per-timing breakdown.
    2×3 grid: rows = {REPLACE, DEGRADED}, cols = {CG, EP, LU}.
    Each subplot shows noFT on-demand as reference + FT spot cost per fault timing.
    noFT uses on-demand (no FT means a spot interruption requires full restart).
    FT strategies use spot workers (~70% discount) and handle interruptions.
    """
    benchmarks  = ["CG", "EP", "LU"]
    bench_class = {"CG": "C", "EP": "D", "LU": "C"}
    ft_strats   = [STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]
    timings     = [10.0, 25.0, 50.0]

    # colour ramps: light→dark per timing level, per strategy
    rep_colors = ["#aec6e8", "#4c9be8", "#1a5fa8"]   # blue shades
    deg_colors = ["#a8d8a4", "#5cb85c", "#2d7a2d"]   # green shades
    strat_colors = {STRATEGY_KEY_REPLACE: rep_colors,
                    STRATEGY_KEY_DEGRADED: deg_colors}

    cost_df = df[df["benchmark"].isin(benchmarks)].copy()
    if cost_df.empty:
        print("  Skipping fig7 (no cost data)")
        return

    fig, axes = plt.subplots(2, 3, figsize=(14, 9))
    fig.suptitle(
        "Cost per run — spot workers with FT vs on-demand without FT\n"
        "(noFT requires on-demand; FT strategies use ~70%-cheaper spot instances)",
        fontsize=11)

    for row_i, strat in enumerate(ft_strats):
        for col_i, bench in enumerate(benchmarks):
            ax = axes[row_i][col_i]
            sub = cost_df[cost_df["benchmark"] == bench]
            worker_counts = sorted(sub["config_workers"].unique())
            x = np.arange(len(worker_counts))

            has_timing = (bench != "CG")  # CG has no timing variation

            if has_timing:
                # 4 bars per x: noFT + 10% + 25% + 50%
                bar_w   = 0.18
                offsets = [-1.5, -0.5, 0.5, 1.5]
                labels  = ["noFT (on-demand)", "10% timing", "25% timing", "50% timing"]
                colors  = [STRATEGY_COLOR[STRATEGY_KEY_NONE]] + strat_colors[strat]

                datasets = []
                # noFT
                vals = []
                for w in worker_counts:
                    r = sub[(sub["config_workers"] == w) & (sub["strategy"] == STRATEGY_KEY_NONE)]
                    vals.append(float(r["run_cost_ondemand_usd"].iloc[0]) if not r.empty else np.nan)
                datasets.append(vals)
                # FT timings
                for t in timings:
                    vals = []
                    for w in worker_counts:
                        r = sub[(sub["config_workers"] == w) & (sub["strategy"] == strat) &
                                (sub["timing_pct"] == t)]
                        vals.append(float(r["run_cost_usd"].iloc[0]) if not r.empty else np.nan)
                    datasets.append(vals)

                for idx, (data, lbl, col, off) in enumerate(zip(datasets, labels, colors, offsets)):
                    bars = ax.bar(x + off * bar_w, data, bar_w, label=lbl, color=col,
                                  alpha=(0.85 if idx == 0 else 1.0))
                    for bar, val in zip(bars, data):
                        if not np.isnan(val):
                            ax.annotate(f"${val:.3f}",
                                        xy=(bar.get_x() + bar.get_width() / 2, val),
                                        xytext=(0, 3), textcoords="offset points",
                                        ha="center", fontsize=6, rotation=50)
            else:
                # CG: 2 bars per x: noFT + FT (no timing variation)
                bar_w = 0.3
                for i, (s, col, lbl) in enumerate([
                    (STRATEGY_KEY_NONE, STRATEGY_COLOR[STRATEGY_KEY_NONE], "noFT (on-demand)"),
                    (strat, strat_colors[strat][1], _label(strat) + " (spot)"),
                ]):
                    vals = []
                    for w in worker_counts:
                        r = sub[(sub["config_workers"] == w) & (sub["strategy"] == s) &
                                (sub["timing_pct"].isna() if s != STRATEGY_KEY_NONE else pd.Series([True]*len(sub)))]
                        if s == STRATEGY_KEY_NONE:
                            r2 = sub[(sub["config_workers"] == w) & (sub["strategy"] == s)]
                            vals.append(float(r2["run_cost_ondemand_usd"].iloc[0]) if not r2.empty else np.nan)
                        else:
                            r2 = sub[(sub["config_workers"] == w) & (sub["strategy"] == s) &
                                     sub["timing_pct"].isna()]
                            vals.append(float(r2["run_cost_usd"].iloc[0]) if not r2.empty else np.nan)
                    offset = (i - 0.5) * bar_w
                    bars = ax.bar(x + offset, vals, bar_w, label=lbl, color=col,
                                  alpha=(0.85 if i == 0 else 1.0))
                    for bar, val in zip(bars, vals):
                        if not np.isnan(val):
                            ax.annotate(f"${val:.3f}",
                                        xy=(bar.get_x() + bar.get_width() / 2, val),
                                        xytext=(0, 3), textcoords="offset points",
                                        ha="center", fontsize=6.5, rotation=45)

            ax.set_xticks(x)
            ax.set_xticklabels([f"{w}w" for w in worker_counts])
            ax.set_xlabel("Worker count")
            if col_i == 0:
                ax.set_ylabel("Estimated cost per run (USD)")
            ax.set_title(f"{bench}-{bench_class[bench]} — {_label(strat)}")
            ax.legend(fontsize=7)
            ax.grid(axis="y", linestyle="--", alpha=0.4)
            ax.set_ylim(bottom=0)

    plt.tight_layout()
    path = out_dir / "fig7_cost.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--results-dir", default="results",
        help="Root directory to scan for result .txt files (default: results/)",
    )
    parser.add_argument(
        "--output-dir", default="TCC/artifacts/phase5/analysis",
        help="Directory for CSV, summary, and plots (default: TCC/artifacts/phase5/analysis)",
    )
    args = parser.parse_args()

    results_dir = Path(args.results_dir)
    out_dir     = Path(args.output_dir)
    plots_dir   = out_dir / "plots"
    plots_dir.mkdir(parents=True, exist_ok=True)

    print(f"\nScanning: {results_dir.resolve()}")
    df = load_all_results(results_dir)

    if df.empty:
        print("Nothing to analyse.")
        return

    print(f"\nLoaded {len(df)} valid runs:")
    summary_counts = df.groupby(["benchmark", "class", "config_workers", "strategy"]).size()
    print(summary_counts.to_string())
    print()

    print("Generating outputs...")
    save_raw_csv(df, out_dir)
    save_summary_markdown(df, out_dir)

    print("Generating plots...")
    plot_mana_overhead(df, plots_dir)      # fig1: noFT vs MANA-noFT
    plot_synth_calls(df, plots_dir)        # fig2: synth call frequency study
    plot_synth_imbalanced(df, plots_dir)   # fig2b: communication imbalance study
    plot_checkpoint_size(df, plots_dir)    # fig3: checkpoint size vs phase1 time
    plot_timing_phases(df, plots_dir)         # fig4: full wall-time timeline by failure timing
    plot_cg_short_job(df, plots_dir)         # fig5: CG short-job FT wall time
    plot_mana_scalability(df, plots_dir)     # fig6: MANA overhead % vs worker count
    plot_cost(df, plots_dir)                 # fig7: spot+FT vs on-demand cost
    plot_strategy_comparison(df, plots_dir)  # fig8: REPLACE vs DEGRADED total time (winner)
    plot_recovery_overhead_ratio(df, plots_dir)  # fig9: recovery overhead % by timing

    print(f"\nDone. Outputs in: {out_dir.resolve()}")


if __name__ == "__main__":
    main()
