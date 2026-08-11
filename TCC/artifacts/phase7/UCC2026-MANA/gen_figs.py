#!/usr/bin/env python3
"""
UCC 2026 paper figure generator.

Reads TCC/artifacts/phase5/analysis/results_raw.csv and saves
paper-ready figures to TCC/artifacts/phase7/UCC2026-MANA/imgs/.

Run from repository root:
  python TCC/artifacts/phase7/UCC2026-MANA/gen_figs.py

Figures produced:
  fig_overhead.png      -- MANA overhead vs noFT baseline  (figure*, textwidth)
  fig_savings.png       -- Cost savings % spot+DEGRADED vs noFT on-demand  (figure*, textwidth)
  fig_crossover.png     -- Strategy selection heatmap: Delta T_Replace - T_Degraded  (figure, columnwidth)
  fig_ckpt_latency.png  -- Checkpoint write time vs per-process memory footprint  (figure, columnwidth)

Design principle: figsize equals intended PDF display size, so matplotlib
point sizes translate 1-to-1 to PDF point sizes without rescaling guesswork.
"""

from pathlib import Path
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.colors import TwoSlopeNorm
from matplotlib.patches import Patch

# ── Layout (IEEE IEEEtran two-column, US letter) ─────────────────────────────
COLWIDTH  = 3.4    # single column, inches
TEXTWIDTH = 7.0    # both columns + gutter, inches
DPI       = 300

CSV_PATH = Path("TCC/artifacts/phase5/analysis/results_raw.csv")
OUT_DIR  = Path("TCC/artifacts/phase7/UCC2026-MANA/imgs")

# ── Strategy keys ─────────────────────────────────────────────────────────────
NONE      = "noFT"
MANA_NONE = "MANA_noFT"
REPLACE   = "REPLACE"
DEGRADED  = "DEGRADED"

COLORS = {
    NONE:      "#4C72B0",
    MANA_NONE: "#55A868",
    REPLACE:   "#DD8452",
    DEGRADED:  "#C44E52",
}

BENCH_CLASS = {"CG": "C", "EP": "D", "LU": "C"}


def ms(rows, col):
    """Return (mean, std) of col; std = 0 for n <= 1."""
    v = rows[col].dropna()
    if len(v) == 0:
        return np.nan, 0.0
    return float(v.mean()), float(v.std(ddof=1)) if len(v) > 1 else 0.0


def set_style():
    """Apply rcParams sized for the intended display dimensions."""
    plt.rcParams.update({
        "font.family":          "sans-serif",
        "font.size":            8,
        "axes.titlesize":       8,
        "axes.labelsize":       7.5,
        "xtick.labelsize":      7,
        "ytick.labelsize":      7,
        "legend.fontsize":      6.5,
        "legend.handlelength":  1.2,
        "legend.handletextpad": 0.4,
        "legend.columnspacing": 0.8,
        "legend.borderpad":     0.3,
        "lines.linewidth":      1.2,
        "axes.linewidth":       0.75,
        "xtick.major.size":     2.5,
        "ytick.major.size":     2.5,
        "xtick.major.width":    0.75,
        "ytick.major.width":    0.75,
    })


# ── Figure 1: MANA overhead ───────────────────────────────────────────────────

def fig_mana_overhead(df: pd.DataFrame, out_dir: Path):
    """
    3 panels (CG-C, EP-D, LU-C): noFT vs MANA-noFT elapsed time per worker count.
    Overhead percentage annotated above each MANA bar.
    Layout: figure* at textwidth.
    Message: MANA overhead is moderate for EP-D/LU-C; CG-C is an anomaly
             because its 5-second execution time magnifies fixed startup costs.
    """
    data = df[
        df["benchmark"].isin(["CG", "EP", "LU"]) &
        df["strategy"].isin([NONE, MANA_NONE])
    ].copy()

    fig, axes = plt.subplots(1, 3, figsize=(TEXTWIDTH, 2.4), sharey=False)
    ek = {"elinewidth": 0.75, "ecolor": "black"}

    for ax, bench in zip(axes, ["CG", "EP", "LU"]):
        sub     = data[data["benchmark"] == bench]
        workers = sorted(sub["config_workers"].unique())
        x       = np.arange(len(workers))
        bw      = 0.35

        noft_m, noft_e, mana_m, mana_e = [], [], [], []
        for w in workers:
            nm, ne = ms(sub[(sub["config_workers"] == w) & (sub["strategy"] == NONE)],
                        "ft_wall_time_s")
            mm, me = ms(sub[(sub["config_workers"] == w) & (sub["strategy"] == MANA_NONE)],
                        "ft_wall_time_s")
            noft_m.append(nm); noft_e.append(ne)
            mana_m.append(mm); mana_e.append(me)

        ax.bar(x - bw / 2, noft_m, bw, yerr=noft_e, capsize=2,
               label="noFT", color=COLORS[NONE], error_kw=ek)
        ax.bar(x + bw / 2, mana_m, bw, yerr=mana_e, capsize=2,
               label="MANA (no failure)", color=COLORS[MANA_NONE], error_kw=ek)

        for i, (n, m) in enumerate(zip(noft_m, mana_m)):
            if not (np.isnan(n) or np.isnan(m)) and n > 0:
                pct = (m - n) / n * 100
                ax.annotate(f"+{pct:.0f}%",
                            xy=(x[i] + bw / 2, m),
                            xytext=(0, 2.5), textcoords="offset points",
                            ha="center", fontsize=6, color="#1a5c1a", fontweight="bold")

        ax.set_title(f"{bench}-{BENCH_CLASS[bench]}")
        ax.set_xticks(x)
        ax.set_xticklabels([f"{w}w" for w in workers])
        ax.set_xlabel("Workers")
        if bench == "CG":
            ax.set_ylabel("Elapsed time (s)")
        ax.legend(loc="upper right", framealpha=0.85, borderaxespad=0.3)
        ax.grid(axis="y", linestyle="--", alpha=0.35, linewidth=0.5)
        ax.set_ylim(bottom=0)

    plt.tight_layout(pad=0.4, w_pad=0.8)
    path = out_dir / "fig_overhead.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight")
    plt.close()
    print(f"  Saved: {path}")


# ── Figure 2: Cost savings ────────────────────────────────────────────────────

def fig_cost_savings(df: pd.DataFrame, out_dir: Path):
    """
    2 panels (EP-D, LU-C): cost savings % of DEGRADED-spot vs noFT-on-demand.
    CG-C is excluded: its 5-second execution time is far shorter than recovery overhead,
    so it always costs more with FT than without -- this is noted in the text, not plotted.
    Positive savings = spot+FT is cheaper.  Dashed line at 0 marks break-even.
    Three grouped bars per cluster size: one per failure timing (10 / 25 / 50%).
    Layout: figure* at textwidth.
    """
    fdata   = df[df["benchmark"].isin(["EP", "LU"])].copy()
    timings = [10.0, 25.0, 50.0]

    tim_colors = ["#a8d8a4", "#5cb85c", "#2d7a2d"]
    tim_labels = ["Failure at 10%", "Failure at 25%", "Failure at 50%"]

    fig, axes = plt.subplots(1, 2, figsize=(TEXTWIDTH * 0.72, 2.5), sharey=False)

    for ax, bench in zip(axes, ["EP", "LU"]):
        sub           = fdata[fdata["benchmark"] == bench]
        worker_counts = sorted(sub["config_workers"].unique())
        x             = np.arange(len(worker_counts))
        bw            = 0.22
        offsets       = [-0.22, 0.0, 0.22]

        ax.axhline(0, color="black", linewidth=0.8, linestyle="--", zorder=0)

        for t_idx, (t, col, lbl) in enumerate(zip(timings, tim_colors, tim_labels)):
            savings_m = []
            for w in worker_counts:
                noft_rows = sub[(sub["config_workers"] == w) & (sub["strategy"] == NONE)]
                deg_rows  = sub[(sub["config_workers"] == w) &
                                (sub["strategy"] == DEGRADED) &
                                (sub["timing_pct"] == t)]
                noft_m, _ = ms(noft_rows, "run_cost_ondemand_usd")
                deg_m, _  = ms(deg_rows,  "run_cost_usd")
                if np.isnan(noft_m) or noft_m == 0 or np.isnan(deg_m):
                    savings_m.append(np.nan)
                else:
                    savings_m.append((noft_m - deg_m) / noft_m * 100)

            bars = ax.bar(x + offsets[t_idx], savings_m, bw,
                          label=lbl, color=col, zorder=2)

            # Annotate each bar with its value
            for bar, val in zip(bars, savings_m):
                if not np.isnan(val):
                    va = "bottom" if val >= 0 else "top"
                    offset_y = 0.3 if val >= 0 else -0.3
                    ax.annotate(f"{val:.0f}%",
                                xy=(bar.get_x() + bar.get_width() / 2,
                                    val + offset_y),
                                ha="center", va=va, fontsize=5.5, color="#333")

        ax.set_title(f"{bench}-{BENCH_CLASS[bench]}")
        ax.set_xticks(x)
        ax.set_xticklabels([f"{w}w" for w in worker_counts])
        ax.set_xlabel("Workers")
        if bench == "EP":
            ax.set_ylabel("Cost savings vs. noFT on-demand (%)")
        ax.legend(loc="lower right", framealpha=0.85, borderaxespad=0.3)
        ax.grid(axis="y", linestyle="--", alpha=0.35, linewidth=0.5)
        ymin, ymax = ax.get_ylim()
        ax.set_ylim(min(ymin, -2), max(ymax * 1.12, 5))

    plt.tight_layout(pad=0.4, w_pad=0.9)
    path = out_dir / "fig_savings.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight")
    plt.close()
    print(f"  Saved: {path}")


# ── Figure 3: Strategy crossover heatmap ──────────────────────────────────────

def fig_crossover(df: pd.DataFrame, out_dir: Path):
    """
    Two heatmaps (EP-D left, LU-C right): Delta = T_Replace - T_Degraded (seconds).
    Positive (green) = DEGRADED completes sooner.
    Negative (red, outlined) = REPLACE completes sooner.
    Layout: figure at columnwidth.
    Message: cluster size is the dominant factor; DEGRADED wins 17 of 18 configurations;
             the single REPLACE-wins cell (EP-D, 2w, 10%) is marked.
    """
    data = df[
        df["benchmark"].isin(["EP", "LU"]) &
        df["strategy"].isin([REPLACE, DEGRADED]) &
        df["timing_pct"].notna()
    ].copy()

    pcts        = [10, 25, 50]
    workers     = [2, 4, 8]
    benchmarks  = ["EP", "LU"]
    bench_label = {"EP": "EP-D", "LU": "LU-C"}

    matrices = {}
    for bench in benchmarks:
        mat = np.full((len(pcts), len(workers)), np.nan)
        for r, pct in enumerate(pcts):
            for c, N in enumerate(workers):
                rep = data[(data["benchmark"] == bench) & (data["config_workers"] == N) &
                           (data["strategy"] == REPLACE) & (data["timing_pct"] == pct)]
                deg = data[(data["benchmark"] == bench) & (data["config_workers"] == N) &
                           (data["strategy"] == DEGRADED) & (data["timing_pct"] == pct)]
                if rep.empty or deg.empty:
                    continue
                mat[r, c] = ms(rep, "ft_wall_time_s")[0] - ms(deg, "ft_wall_time_s")[0]
        matrices[bench] = mat

    all_vals = np.concatenate([matrices[b].flatten() for b in benchmarks])
    valid    = all_vals[~np.isnan(all_vals)]
    norm     = TwoSlopeNorm(vmin=valid.min(), vcenter=0.0, vmax=valid.max())

    fig, axes = plt.subplots(1, 2, figsize=(COLWIDTH, 1.85),
                             gridspec_kw={"wspace": 0.40})

    for ax, bench in zip(axes, benchmarks):
        mat = matrices[bench]
        ax.imshow(mat, cmap="RdYlGn", norm=norm,
                  aspect="auto", origin="upper", interpolation="nearest")

        for r in range(len(pcts)):
            for c in range(len(workers)):
                val = mat[r, c]
                if np.isnan(val):
                    continue
                fg   = "white" if abs(val) > 60 else "black"
                sign = "+" if val >= 0 else ""
                ax.text(c, r, f"{sign}{int(round(val))}s",
                        ha="center", va="center",
                        fontsize=6.5, fontweight="bold", color=fg)
                if val < 0:
                    ax.add_patch(plt.Rectangle(
                        (c - 0.5, r - 0.5), 1, 1,
                        fill=False, edgecolor="#7a1a1a",
                        linewidth=2.0, zorder=4))

        ax.set_xticks(range(len(workers)))
        ax.set_xticklabels([f"{w}w" for w in workers])
        ax.set_yticks(range(len(pcts)))
        ax.set_yticklabels([f"{p}%" for p in pcts])
        ax.set_xlabel("Workers ($N$)")
        if bench == "EP":
            ax.set_ylabel("Failure timing")
        ax.set_title(bench_label[bench])

    legend_handles = [
        Patch(facecolor="#4daf4a", label="DEGRADED faster"),
        Patch(facecolor="#e41a1c", label="REPLACE faster"),
    ]
    fig.legend(handles=legend_handles, loc="lower center", ncol=2,
               fontsize=6.5, bbox_to_anchor=(0.47, -0.18),
               framealpha=0.9, handlelength=1.0, handleheight=0.8)

    plt.tight_layout(pad=0.4)
    path = out_dir / "fig_crossover.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight")
    plt.close()
    print(f"  Saved: {path}")


# ── Figure 4: Checkpoint write latency ───────────────────────────────────────

def fig_ckpt_latency(out_dir: Path):
    """
    Phase-1 (checkpoint write) duration vs per-process memory footprint.
    Data from synth_checkpoint_size (4 footprint levels, N=3 each).
    Linear axes + regression line show linear scaling.
    Horizontal reference at 120 s marks the two-minute termination window.
    Layout: figure at columnwidth.
    """
    xs    = np.array([50, 200, 800, 3200], dtype=float)
    means = np.array([16.4, 26.9, 47.0, 139.2])
    stds  = np.array([0.1,   0.4,  0.2,   0.4])

    coeffs = np.polyfit(xs, means, 1)
    x_fit  = np.linspace(0, xs.max() * 1.08, 300)
    y_fit  = np.polyval(coeffs, x_fit)

    fig, ax = plt.subplots(figsize=(COLWIDTH, 2.4))

    ax.plot(x_fit, y_fit, "--", color="#DD8452", linewidth=1.0,
            label=f"Linear fit ({coeffs[0]*1000:.1f} ms/MB)")

    ax.errorbar(xs, means, yerr=stds, fmt="o", color="#4C72B0",
                capsize=3, markersize=4, linewidth=0,
                elinewidth=0.75, ecolor="black",
                label="Phase 1", zorder=5)

    ax.axhline(120, color="#C44E52", linewidth=0.9, linestyle=":",
               label="2-min window")

    labels  = ["50 MB", "200 MB", "800 MB", "3 200 MB"]
    offsets = [(30, 3), (30, 3), (30, 3), (-380, 3)]
    for x, y, lbl, (dx, dy) in zip(xs, means, labels, offsets):
        ax.annotate(lbl, xy=(x, y), xytext=(x + dx, y + dy), fontsize=6)

    ax.set_xlabel("Memory per MPI process (MB)")
    ax.set_ylabel("Phase 1 (s)")
    ax.set_xlim(left=0)
    ax.set_ylim(bottom=0)
    ax.legend(loc="upper left", framealpha=0.85, borderaxespad=0.3)
    ax.grid(linestyle="--", alpha=0.35, linewidth=0.5)

    plt.tight_layout(pad=0.4)
    path = out_dir / "fig_ckpt_latency.png"
    plt.savefig(path, dpi=DPI, bbox_inches="tight")
    plt.close()
    print(f"  Saved: {path}")


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    set_style()
    print(f"Reading: {CSV_PATH}")
    df = pd.read_csv(CSV_PATH)
    print(f"  {len(df)} rows")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    print(f"Output:  {OUT_DIR}\n")

    print("Generating figures...")
    fig_mana_overhead(df, OUT_DIR)
    fig_cost_savings(df, OUT_DIR)
    fig_crossover(df, OUT_DIR)
    fig_ckpt_latency(OUT_DIR)
    print("\nDone.")


if __name__ == "__main__":
    main()
