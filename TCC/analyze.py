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
    Parse a task_tag like 'lu-C_4w_m5xl_replace' into components.
    Returns dict with: benchmark, class, config_workers, strategy_suffix.
    """
    m = re.match(r"^([a-zA-Z]+)-([A-Z])_(\d+)w_[^_]+_(.+)$", tag)
    if m:
        return {
            "benchmark":       m.group(1).upper(),
            "class":           m.group(2),
            "config_workers":  int(m.group(3)),
            "strategy_suffix": m.group(4),
        }
    return {"benchmark": tag, "class": "?", "config_workers": 0, "strategy_suffix": "?"}


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

        # Aggregate FT phase metrics across all valid cycles
        # (discard no_cycle_id entries — those are warning-only cycles)
        valid_cycles = [
            v for k, v in ft_metrics_by_cycle.items()
            if k != "no_cycle_id"
            and "phase1_detection_to_checkpoint_s" in v
        ]
        phase1 = phase2 = phase3 = recovery_total = None
        if valid_cycles:
            def safe_float(d, k):
                try: return float(d.get(k, ""))
                except (ValueError, TypeError): return None

            p1s = [safe_float(c, "phase1_detection_to_checkpoint_s") for c in valid_cycles]
            p2s = [safe_float(c, "phase2_checkpoint_to_dispatch_s")  for c in valid_cycles]
            p3s = [safe_float(c, "phase3_dispatch_to_done_s")         for c in valid_cycles]
            rs  = [safe_float(c, "total_recovery_s")                  for c in valid_cycles]

            def mean_or_none(lst):
                vals = [x for x in lst if x is not None]
                return sum(vals) / len(vals) if vals else None

            phase1         = mean_or_none(p1s)
            phase2         = mean_or_none(p2s)
            phase3         = mean_or_none(p3s)
            recovery_total = mean_or_none(rs)

        run_cost, run_cost_od, total_cost = compute_cost({**rm})

        # auto_failure_trigger_secs — present only for FT runs with auto trigger
        raw_trigger = rm.get("auto_failure_trigger_secs")
        auto_trigger_secs = int(raw_trigger) if raw_trigger is not None else None

        record = {
            # Identity
            "source_file":               filepath.name,
            "task_tag":                  rm.get("task_tag", ""),
            "benchmark":                 tag_parts.get("benchmark", "?"),
            "class":                     tag_parts.get("class", "?"),
            "config_workers":            tag_parts.get("config_workers", 0),
            "strategy":                  strategy_key,
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
            "recovery_cycles":      len(valid_cycles) if valid_cycles else 0,
            "phase1_s":             phase1,
            "phase2_s":             phase2,
            "phase3_s":             phase3,
            "recovery_total_s":     recovery_total,
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
        "phase1_s", "phase2_s", "phase3_s", "recovery_total_s",
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
                           "FT Wall Time (s)", "Mop/s total",
                           "Phase1 (s)", "Phase2 (s)", "Phase3 (s)",
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
                # Trigger: show value if consistent, flag if it varied across runs
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
                    fmt("phase1_s_mean",       "phase1_s_std",       decimals=1),
                    fmt("phase2_s_mean",       "phase2_s_std",       decimals=1),
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

def _grouped_bar(ax, data_by_group, group_labels, series_keys,
                 series_labels, series_colors,
                 errors_by_group=None, ylabel="", title=""):
    """
    Draw a grouped bar chart.
    data_by_group:   dict { group_label -> { series_key -> value } }
    errors_by_group: dict { group_label -> { series_key -> std } } or None
    """
    n_groups  = len(group_labels)
    n_series  = len(series_keys)
    if n_series == 0:
        return
    bar_width = 0.7 / n_series
    x         = np.arange(n_groups)

    for i, (sk, sl, sc) in enumerate(zip(series_keys, series_labels, series_colors)):
        vals  = [data_by_group.get(g, {}).get(sk, np.nan) for g in group_labels]
        yerrs = None
        if errors_by_group:
            yerrs = [errors_by_group.get(g, {}).get(sk, 0) or 0 for g in group_labels]
        offset = (i - n_series / 2 + 0.5) * bar_width
        ax.bar(x + offset, vals, bar_width, label=sl, color=sc,
               yerr=yerrs, capsize=3, error_kw={"elinewidth": 1})

    ax.set_xticks(x)
    ax.set_xticklabels(group_labels)
    ax.set_ylabel(ylabel)
    ax.set_title(title)
    ax.legend(fontsize=8)
    ax.yaxis.set_minor_locator(mticker.AutoMinorLocator())
    ax.grid(axis="y", linestyle="--", alpha=0.4)


# ── Plots ────────────────────────────────────────────────────────────────────

def plot_wall_time(df: pd.DataFrame, out_dir: Path):
    """Fig 1: FT Wall Time by workers, grouped by strategy — one subplot per benchmark."""
    benchmarks = sorted(df["benchmark"].unique())
    fig, axes = plt.subplots(1, len(benchmarks), figsize=(6 * len(benchmarks), 5), squeeze=False)
    fig.suptitle("FT Wall Time — LU/EP/CG Class C (m5.xlarge, 2 proc/node)", fontsize=12)

    for ax, bench in zip(axes[0], benchmarks):
        sub = df[df["benchmark"] == bench]
        worker_counts = sorted(sub["config_workers"].unique())
        strategies = sorted(sub["strategy"].unique(), key=_strat_order)

        data, errs = defaultdict(dict), defaultdict(dict)
        for w in worker_counts:
            for s in strategies:
                vals = sub[(sub["config_workers"] == w) & (sub["strategy"] == s)]["ft_wall_time_s"]
                if len(vals):
                    data[f"{w}w"][s] = vals.mean()
                    errs[f"{w}w"][s] = vals.std() if len(vals) > 1 else 0

        group_labels = [f"{w}w" for w in worker_counts]
        _grouped_bar(
            ax, data, group_labels,
            strategies, [_label(s) for s in strategies], [_color(s) for s in strategies],
            errors_by_group=errs,
            ylabel="Wall Time (s)", title=bench,
        )

    plt.tight_layout()
    path = out_dir / "fig1_wall_time.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_mana_overhead(df: pd.DataFrame, out_dir: Path):
    """
    Fig 2: Total overhead relative to noFT (raw MPI) baseline.
    All 4 strategies shown as bars — noFT bar = 1.0× (the reference).
    Answers: "How much total overhead does each FT approach add vs running
    without any fault tolerance?"
      - MANA noFT bar  → pure MANA interposition cost (no failures)
      - REPLACE bar    → MANA cost + time to reprovision a new node
      - DEGRADED bar   → MANA cost + faster degraded restart
    """
    benchmarks = sorted(df["benchmark"].unique())
    fig, axes = plt.subplots(1, len(benchmarks), figsize=(6 * len(benchmarks), 5), squeeze=False)
    fig.suptitle("Total Overhead vs noFT baseline  (1.0 = same speed as raw MPI)", fontsize=12)

    for ax, bench in zip(axes[0], benchmarks):
        sub = df[df["benchmark"] == bench]
        worker_counts = sorted(sub["config_workers"].unique())
        strategies = [s for s in STRATEGY_ORDER if s in sub["strategy"].values]

        data, errs = defaultdict(dict), defaultdict(dict)
        for w in worker_counts:
            noft_vals = sub[
                (sub["config_workers"] == w) & (sub["strategy"] == STRATEGY_KEY_NONE)
            ]["ft_wall_time_s"]
            baseline = noft_vals.mean() if len(noft_vals) else None
            if baseline is None:
                continue
            for s in strategies:
                vals = sub[(sub["config_workers"] == w) & (sub["strategy"] == s)]["ft_wall_time_s"]
                if len(vals):
                    data[f"{w}w"][s] = vals.mean() / baseline
                    errs[f"{w}w"][s] = (vals.std() / baseline) if len(vals) > 1 else 0

        group_labels = [f"{w}w" for w in worker_counts if f"{w}w" in data]
        if not group_labels:
            ax.set_title(f"{bench} (no data)")
            continue
        _grouped_bar(
            ax, data, group_labels,
            strategies, [_label(s) for s in strategies], [_color(s) for s in strategies],
            errors_by_group=errs,
            ylabel="Overhead factor (× noFT)", title=bench,
        )
        ax.yaxis.set_minor_locator(mticker.AutoMinorLocator())

    plt.tight_layout()
    path = out_dir / "fig2_mana_overhead.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_recovery_phases(df: pd.DataFrame, out_dir: Path):
    """Fig 3: Stacked bar of Phase1/Phase2/Phase3 for FT runs."""
    ft_df = df[df["strategy"].isin([STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED])].copy()
    if ft_df.empty:
        print("  Skipping fig3 (no FT runs)")
        return

    benchmarks = sorted(ft_df["benchmark"].unique())
    fig, axes = plt.subplots(1, len(benchmarks), figsize=(6 * len(benchmarks), 5), squeeze=False)
    fig.suptitle("Recovery Phase Breakdown (FT runs)", fontsize=12)
    phase_colors = {"Phase 1\n(detect→ckpt)": "#5B9BD5",
                    "Phase 2\n(ckpt→dispatch)": "#ED7D31",
                    "Phase 3\n(dispatch→done)": "#70AD47"}

    for ax, bench in zip(axes[0], benchmarks):
        sub = ft_df[ft_df["benchmark"] == bench]
        worker_counts = sorted(sub["config_workers"].unique())
        strategies    = sorted(sub["strategy"].unique(), key=_strat_order)

        labels = []
        p1s, p2s, p3s = [], [], []

        for w in worker_counts:
            for s in strategies:
                vals = sub[(sub["config_workers"] == w) & (sub["strategy"] == s)]
                if len(vals):
                    labels.append(f"{w}w\n{_label(s)}")
                    p1s.append(vals["phase1_s"].mean())
                    p2s.append(vals["phase2_s"].mean())
                    p3s.append(vals["phase3_s"].mean())

        x = np.arange(len(labels))
        bar_width = 0.5
        cols = list(phase_colors.values())
        b1 = ax.bar(x, p1s, bar_width, label="Phase 1\n(detect→ckpt)",   color=cols[0])
        b2 = ax.bar(x, p2s, bar_width, bottom=p1s, label="Phase 2\n(ckpt→dispatch)", color=cols[1])
        b3 = ax.bar(x, p3s, bar_width,
                    bottom=[a + b for a, b in zip(p1s, p2s)],
                    label="Phase 3\n(dispatch→done)", color=cols[2])

        ax.set_xticks(x)
        ax.set_xticklabels(labels, fontsize=8)
        ax.set_ylabel("Time (s)")
        ax.set_title(bench)
        ax.legend(fontsize=7)
        ax.grid(axis="y", linestyle="--", alpha=0.4)

    plt.tight_layout()
    path = out_dir / "fig3_recovery_phases.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_scalability(df: pd.DataFrame, out_dir: Path):
    """Fig 4: Scalability — wall time vs worker count, one line per strategy."""
    benchmarks = sorted(df["benchmark"].unique())
    fig, axes = plt.subplots(1, len(benchmarks), figsize=(6 * len(benchmarks), 5), squeeze=False)
    fig.suptitle("Scalability: Wall Time vs Worker Count", fontsize=12)

    for ax, bench in zip(axes[0], benchmarks):
        sub = df[df["benchmark"] == bench]
        strategies = sorted(sub["strategy"].unique(), key=_strat_order)
        worker_counts = sorted(sub["config_workers"].unique())

        for s in strategies:
            xs, ys, yerrs = [], [], []
            for w in worker_counts:
                vals = sub[(sub["config_workers"] == w) & (sub["strategy"] == s)]["ft_wall_time_s"]
                if len(vals):
                    xs.append(w)
                    ys.append(vals.mean())
                    yerrs.append(vals.std() if len(vals) > 1 else 0)
            if xs:
                ax.errorbar(xs, ys, yerr=yerrs, label=_label(s),
                            color=_color(s), marker="o", capsize=4,
                            linewidth=1.5, markersize=5)

        ax.set_xlabel("Workers")
        ax.set_ylabel("Wall Time (s)")
        ax.set_title(bench)
        ax.set_xticks(worker_counts)
        ax.legend(fontsize=8)
        ax.grid(linestyle="--", alpha=0.4)

    plt.tight_layout()
    path = out_dir / "fig4_scalability.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_cost(df: pd.DataFrame, out_dir: Path):
    """
    Fig 5: Cost per run — the core economic argument.

    noFT (native):  all nodes on-demand (no FT → can't afford spot interruptions)
    REPLACE/DEGRADED: workers on spot + head on-demand (FT enables cheaper workers)

    MANA_noFT is excluded — it's not a realistic deployment (MANA overhead with
    no benefit), and the comparison of interest is on-demand-noFT vs spot-FT.
    """
    cost_strategies = [STRATEGY_KEY_NONE, STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]

    benchmarks = sorted(df["benchmark"].unique())
    fig, axes = plt.subplots(1, len(benchmarks), figsize=(6 * len(benchmarks), 5), squeeze=False)
    # Short two-line title to avoid clipping
    fig.suptitle("Cost per Run\nnoFT: on-demand workers  |  FT: spot workers", fontsize=11)

    for ax, bench in zip(axes[0], benchmarks):
        sub = df[df["benchmark"] == bench]
        worker_counts = sorted(sub["config_workers"].unique())
        strategies = [s for s in cost_strategies if s in sub["strategy"].values]
        if not strategies:
            ax.set_title(f"{bench} (no cost data)")
            ax.axis("off")
            continue

        data, errs = defaultdict(dict), defaultdict(dict)
        for w in worker_counts:
            for s in strategies:
                # noFT uses on-demand cost; FT strategies use spot-worker cost
                col = "run_cost_ondemand_usd" if s == STRATEGY_KEY_NONE else "run_cost_usd"
                vals = sub[(sub["config_workers"] == w) & (sub["strategy"] == s)][col]
                if len(vals):
                    data[f"{w}w"][s] = vals.mean()
                    errs[f"{w}w"][s] = vals.std() if len(vals) > 1 else 0

        group_labels = [f"{w}w" for w in worker_counts if f"{w}w" in data]
        _grouped_bar(
            ax, data, group_labels,
            strategies, [_label(s) for s in strategies], [_color(s) for s in strategies],
            errors_by_group=errs,
            ylabel="Cost (USD)", title=bench,
        )

    plt.tight_layout()
    path = out_dir / "fig5_cost.png"
    plt.savefig(path, dpi=150)
    plt.close()
    print(f"  Saved: {path}")


def plot_strategy_comparison(df: pd.DataFrame, out_dir: Path):
    """
    Fig 6: Recovery strategy overhead relative to MANA noFT baseline.
    Shows MANA_noFT (= 1.0× reference bar), REPLACE, and DEGRADED.
    Answers: "For someone already using MANA, how much extra wall time
    does each recovery strategy cost when a failure occurs?"
      - MANA noFT bar  → 1.0× (baseline: MANA running without failures)
      - REPLACE bar    → e.g. 1.6× means 60% extra time due to node reprovisioning
      - DEGRADED bar   → e.g. 1.25× means 25% extra time due to degraded restart
    Only MANA-aware strategies are shown — noFT is excluded because it
    cannot recover from failures at all.
    """
    # Need at least MANA_noFT + one FT strategy
    mana_strategies = [STRATEGY_KEY_MANA_NONE, STRATEGY_KEY_REPLACE, STRATEGY_KEY_DEGRADED]
    sub_mana = df[df["strategy"].isin(mana_strategies)]
    if sub_mana.empty:
        print("  Skipping fig6 (no MANA strategy runs)")
        return

    benchmarks = sorted(df["benchmark"].unique())
    fig, axes = plt.subplots(1, len(benchmarks), figsize=(6 * len(benchmarks), 5), squeeze=False)
    fig.suptitle("Recovery Overhead vs MANA noFT  (1.0 = no failure occurred)", fontsize=12)

    for ax, bench in zip(axes[0], benchmarks):
        sub_all = df[df["benchmark"] == bench]
        worker_counts = sorted(sub_all["config_workers"].unique())
        strategies = [s for s in mana_strategies if s in sub_all["strategy"].values]

        data, errs = defaultdict(dict), defaultdict(dict)
        for w in worker_counts:
            baseline_vals = sub_all[
                (sub_all["config_workers"] == w) & (sub_all["strategy"] == STRATEGY_KEY_MANA_NONE)
            ]["ft_wall_time_s"]
            baseline = baseline_vals.mean() if len(baseline_vals) else None
            if baseline is None:
                continue
            for s in strategies:
                vals = sub_all[(sub_all["config_workers"] == w) & (sub_all["strategy"] == s)]["ft_wall_time_s"]
                if len(vals):
                    data[f"{w}w"][s] = vals.mean() / baseline
                    errs[f"{w}w"][s] = (vals.std() / baseline) if len(vals) > 1 else 0

        group_labels = [f"{w}w" for w in worker_counts if f"{w}w" in data]
        if not group_labels:
            ax.set_title(f"{bench} (no data)")
            continue
        _grouped_bar(
            ax, data, group_labels,
            strategies, [_label(s) for s in strategies], [_color(s) for s in strategies],
            errors_by_group=errs,
            ylabel="Overhead factor (× MANA noFT)", title=bench,
        )
        ax.yaxis.set_minor_locator(mticker.AutoMinorLocator())

    plt.tight_layout()
    path = out_dir / "fig6_replace_vs_degraded.png"
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
        "--output-dir", default="TCC/artifacts/phase4/analysis",
        help="Directory for CSV, summary, and plots (default: TCC/artifacts/phase4/analysis)",
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
    plot_wall_time(df, plots_dir)
    plot_mana_overhead(df, plots_dir)
    plot_recovery_phases(df, plots_dir)
    plot_scalability(df, plots_dir)
    plot_cost(df, plots_dir)
    plot_strategy_comparison(df, plots_dir)

    print(f"\nDone. Outputs in: {out_dir.resolve()}")


if __name__ == "__main__":
    main()
