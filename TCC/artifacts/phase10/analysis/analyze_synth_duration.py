#!/usr/bin/env python3
"""
Phase 10 - synth_duration analysis: per-point stats, per-N coefficient,
and the two validation figures (boundary scatter + coefficient comparison).

Run from repository root:
  python TCC/artifacts/phase10/analysis/analyze_synth_duration.py
"""

from pathlib import Path
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
from matplotlib.patches import Patch

ANALYSIS_DIR = Path("TCC/artifacts/phase10/analysis")
IMG_DIR = Path("TCC/artifacts/phase10/imgs")
IMG_DIR.mkdir(parents=True, exist_ok=True)

T_SYNTH = 600.0

COLOR_DEGRADED = "#2a78d6"
COLOR_REPLACE = "#eb6834"
COLOR_DEGRADED_ZONE = "#eaf2fb"
COLOR_REPLACE_ZONE = "#fdece3"
TEXT_PRIMARY = "#0b0b0b"
TEXT_SECONDARY = "#52514e"
GRID = "#e1e0d9"

df = pd.read_csv(ANALYSIS_DIR / "synth_duration_raw.csv")
df = df[df["strategy"].isin(["replace", "degraded"])].copy()
df["remaining_secs"] = T_SYNTH * (1 - df["f_pct"] / 100)

# ── Per-point stats ─────────────────────────────────────────────────────────
grp = df.groupby(["n_workers", "f_pct", "strategy"]).agg(
    ft_mean=("ft_wall_time_s", "mean"),
    ft_std=("ft_wall_time_s", "std"),
    p3_mean=("phase3_s", "mean"),
    p3_std=("phase3_s", "std"),
    n_reps=("ft_wall_time_s", "size"),
).reset_index()
grp.to_csv(ANALYSIS_DIR / "synth_duration_points.csv", index=False)

# Pivot to wide (replace vs degraded side by side) per (n_workers, f_pct)
wide = grp.pivot_table(index=["n_workers", "f_pct"],
                        columns="strategy",
                        values=["ft_mean", "ft_std", "p3_mean"]).reset_index()
wide.columns = ["_".join(c).strip("_") for c in wide.columns.to_flat_index()]
wide["remaining_secs"] = T_SYNTH * (1 - wide["f_pct"] / 100)
wide["penalty_s"] = wide["p3_mean_degraded"] - wide["p3_mean_replace"]
wide["coeff"] = wide["penalty_s"] / wide["remaining_secs"]
wide["winner"] = np.where(wide["ft_mean_replace"] < wide["ft_mean_degraded"], "REPLACE", "DEGRADED")
wide["overlap"] = ~(
    (wide["ft_mean_replace"] - wide["ft_std_replace"] > wide["ft_mean_degraded"] + wide["ft_std_degraded"]) |
    (wide["ft_mean_degraded"] - wide["ft_std_degraded"] > wide["ft_mean_replace"] + wide["ft_std_replace"])
)
wide = wide.sort_values(["n_workers", "f_pct"])
wide.to_csv(ANALYSIS_DIR / "synth_duration_wide.csv", index=False)

print(wide[["n_workers", "f_pct", "remaining_secs", "ft_mean_replace", "ft_mean_degraded",
            "coeff", "winner", "overlap"]].to_string(index=False))

# ── Per-N coefficient (mean +/- std across that N's points) ────────────────
coeff_per_n = wide.groupby("n_workers")["coeff"].agg(["mean", "std", "count"]).reset_index()
coeff_per_n.to_csv(ANALYSIS_DIR / "coefficient_per_n.csv", index=False)
print("\nCoefficient per N:")
print(coeff_per_n.to_string(index=False))

# ── Figure 1: boundary scatter (analogous to fig_adaptive_boundary.png) ────
fig, ax = plt.subplots(figsize=(8.8, 4.9))
X_MAX = 620
Y_MIN, Y_MAX = 0.5, 9.8

CROSSOVER = 210  # real data point (f=65%); zone shading and line must agree on this

for n in [2, 4, 8]:
    sub = wide[wide["n_workers"] == n]
    for _, row in sub.iterrows():
        color = COLOR_REPLACE if row["winner"] == "REPLACE" else COLOR_DEGRADED
        marker = "D" if row["overlap"] else "o"
        ax.scatter(row["remaining_secs"], n, marker=marker, s=130,
                   facecolor=color, edgecolor="white", linewidth=1.3, zorder=3)

ax.axvspan(0, CROSSOVER, color=COLOR_DEGRADED_ZONE, zorder=0)
ax.axvspan(CROSSOVER, X_MAX, color=COLOR_REPLACE_ZONE, zorder=0)
ax.axvline(CROSSOVER, color=TEXT_PRIMARY, linestyle="--", linewidth=1.4, zorder=2)
ax.annotate(f"crossover ~ {CROSSOVER}s\n(all three N)", xy=(CROSSOVER, 6.2),
            xytext=(CROSSOVER + 20, 6.2),
            fontsize=9, color=TEXT_PRIMARY, va="center", ha="left", fontweight="bold")

ax.set_xlim(0, X_MAX)
ax.set_ylim(Y_MIN, Y_MAX)
ax.set_yticks([2, 4, 8])
ax.set_xlabel("Estimated remaining work time at the moment of fault (seconds)", fontsize=10.5)
ax.set_ylabel("Cluster size before fault (N workers)", fontsize=10.5)

ax.grid(True, axis="x", color=GRID, linewidth=0.8, zorder=0)
ax.set_axisbelow(True)
for spine in ["top", "right"]:
    ax.spines[spine].set_visible(False)

color_legend = [
    Patch(facecolor=COLOR_REPLACE, label="REPLACE_RESUME faster (mean)"),
    Patch(facecolor=COLOR_DEGRADED, label="DEGRADED_RESUME faster (mean)"),
]
shape_legend = [
    Line2D([0], [0], marker="o", color="w", markerfacecolor="gray", markeredgecolor="white",
           markersize=10, label="Clear winner (no overlap)"),
    Line2D([0], [0], marker="D", color="w", markerfacecolor="gray", markeredgecolor="white",
           markersize=9, label="Ranges overlap (ambiguous)"),
]
leg1 = ax.legend(handles=color_legend, title="Measured winner (color)",
                  loc="upper right", bbox_to_anchor=(0.99, 0.98),
                  fontsize=8.5, title_fontsize=8.5, framealpha=0.95)
ax.add_artist(leg1)
ax.legend(handles=shape_legend, title="Marker shape",
          loc="upper right", bbox_to_anchor=(0.99, 0.78),
          fontsize=8.5, title_fontsize=8.5, framealpha=0.95)

fig.subplots_adjust(top=0.95, right=0.98, left=0.08, bottom=0.12)
fig.savefig(IMG_DIR / "fig_synth_duration_boundary.png", dpi=200)
plt.close(fig)

# ── Figure 2: coefficient comparison, measured (flat) vs theoretical 1/(N-1) ─
fig, ax = plt.subplots(figsize=(7.2, 4.6))
ns = [2, 4, 8]
measured = [coeff_per_n.set_index("n_workers").loc[n, "mean"] for n in ns]
measured_std = [coeff_per_n.set_index("n_workers").loc[n, "std"] for n in ns]
theoretical = [1.0, 1/3, 1/7]

x = np.arange(len(ns))
width = 0.35
ax.bar(x - width/2, measured, width, yerr=measured_std, capsize=4,
       color=COLOR_DEGRADED, label="Measured (synth_duration)", zorder=3)
ax.bar(x + width/2, theoretical, width,
       color="#b0aea6", label="Naive model 1/(N-1)", zorder=3)

ax.set_xticks(x)
ax.set_xticklabels([f"N={n}" for n in ns])
ax.set_ylabel("Penalty coefficient (coef)", fontsize=10.5)
ax.grid(True, axis="y", color=GRID, linewidth=0.8, zorder=0)
ax.set_axisbelow(True)
for spine in ["top", "right"]:
    ax.spines[spine].set_visible(False)
ax.legend(fontsize=9, framealpha=0.95)

fig.subplots_adjust(top=0.95, right=0.97, left=0.12, bottom=0.12)
fig.savefig(IMG_DIR / "fig_coefficient_n_independence.png", dpi=200)
plt.close(fig)

print(f"\nFigures written to {IMG_DIR}/")
