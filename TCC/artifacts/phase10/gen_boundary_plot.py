#!/usr/bin/env python3
"""
Phase 10 - decision boundary plot for the ADAPTIVE heuristic.

Reads TCC/artifacts/phase5/analysis/results_raw.csv and plots the 18
experimental points against the decision boundary remaining_secs = 389s
(equivalent to remaining_secs * 0.28 > 109).

Run from repository root:
  python TCC/artifacts/phase10/gen_boundary_plot.py
"""

from pathlib import Path
import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D
from matplotlib.patches import Patch

CSV_PATH = Path("TCC/artifacts/phase5/analysis/results_raw.csv")
OUT_DIR = Path("TCC/artifacts/phase10/imgs")
OUT_DIR.mkdir(parents=True, exist_ok=True)

COEFF = 0.28
THRESHOLD = 109.0
BOUNDARY = THRESHOLD / COEFF  # 389.3s

# Validated categorical pair (dataviz skill): slot 1 blue, slot 2 orange.
COLOR_DEGRADED = "#2a78d6"
COLOR_REPLACE = "#eb6834"
COLOR_DEGRADED_ZONE = "#eaf2fb"
COLOR_REPLACE_ZONE = "#fdece3"
TEXT_PRIMARY = "#0b0b0b"
TEXT_SECONDARY = "#52514e"
GRID = "#e1e0d9"

BENCH_MARKER = {"EP": "o", "LU": "s"}
BENCH_LABEL = {"EP": "EP-D", "LU": "LU-C"}

df = pd.read_csv(CSV_PATH)
mana_means = df[df["strategy"] == "MANA_noFT"].groupby(
    ["benchmark", "config_workers"]
)["ft_wall_time_s"].mean()

known_winners = {
    ("EP", 2, 10): "REPLACE", ("EP", 2, 25): "DEGRADED", ("EP", 2, 50): "DEGRADED",
    ("EP", 4, 10): "DEGRADED", ("EP", 4, 25): "DEGRADED", ("EP", 4, 50): "DEGRADED",
    ("EP", 8, 10): "DEGRADED", ("EP", 8, 25): "DEGRADED", ("EP", 8, 50): "DEGRADED",
    ("LU", 2, 10): "DEGRADED", ("LU", 2, 25): "DEGRADED", ("LU", 2, 50): "DEGRADED",
    ("LU", 4, 10): "DEGRADED", ("LU", 4, 25): "DEGRADED", ("LU", 4, 50): "DEGRADED",
    ("LU", 8, 10): "DEGRADED", ("LU", 8, 25): "DEGRADED", ("LU", 8, 50): "DEGRADED",
}

fig, ax = plt.subplots(figsize=(8.8, 4.9))

X_MAX = 470
Y_MIN, Y_MAX = 0.5, 9.8

# Shaded decision regions, drawn first (background)
ax.axvspan(0, BOUNDARY, color=COLOR_DEGRADED_ZONE, zorder=0)
ax.axvspan(BOUNDARY, X_MAX, color=COLOR_REPLACE_ZONE, zorder=0)

for bench in ["EP", "LU"]:
    for n in [2, 4, 8]:
        t_mana = mana_means[(bench, n)]
        for f_pct in [10, 25, 50]:
            remaining = t_mana * (1 - f_pct / 100)
            winner = known_winners[(bench, n, f_pct)]
            color = COLOR_REPLACE if winner == "REPLACE" else COLOR_DEGRADED
            ax.scatter(
                remaining, n,
                marker=BENCH_MARKER[bench],
                s=110,
                facecolor=color,
                edgecolor="white",
                linewidth=1.2,
                zorder=3,
            )

# Boundary line + label (short, sits just left of the line, in a row with no markers)
ax.axvline(BOUNDARY, color=TEXT_PRIMARY, linestyle="--", linewidth=1.4, zorder=2)
ax.annotate(
    f"threshold = {BOUNDARY:.0f}s",
    xy=(BOUNDARY, 3.0), xytext=(BOUNDARY - 12, 3.0),
    fontsize=9, color=TEXT_PRIMARY, va="center", ha="right", fontweight="bold",
)

# Zone labels, placed well clear of the boundary and of each other
ax.text(BOUNDARY / 2, Y_MIN + 0.28, "DEGRADED_RESUME wins",
        fontsize=10, color=COLOR_DEGRADED, ha="center", fontweight="bold")
ax.text(BOUNDARY + (X_MAX - BOUNDARY) / 2, Y_MIN + 0.28, "REPLACE_RESUME\nwins",
        fontsize=10, color=COLOR_REPLACE, ha="center", fontweight="bold")

ax.set_xlim(0, X_MAX)
ax.set_ylim(Y_MIN, Y_MAX)
ax.set_yticks([2, 4, 8])
ax.set_xlabel("Estimated remaining work time at the moment of fault (seconds)", fontsize=10.5)
ax.set_ylabel("Cluster size before fault (N workers)", fontsize=10.5)
ax.set_title(
    "REPLACE wins when 0.28 x (remaining work time) > 109s\n"
    "(109s = time REPLACE loses waiting for a new machine)",
    fontsize=9.5, color=TEXT_SECONDARY, pad=10, linespacing=1.6,
)
ax.text(0.0, 1.20, "ADAPTIVE decision boundary: REPLACE vs. DEGRADED",
        transform=ax.transAxes, fontsize=14, fontweight="bold", color=TEXT_PRIMARY,
        ha="left", va="bottom")

ax.grid(True, axis="x", color=GRID, linewidth=0.8, zorder=0)
ax.set_axisbelow(True)
for spine in ["top", "right"]:
    ax.spines[spine].set_visible(False)

# Two separate, unambiguous legends placed in the empty upper-right area
# (all N=8/N=4 data sit at low remaining-time values, so this corner is empty)
shape_legend = [
    Line2D([0], [0], marker=BENCH_MARKER[b], color="w", markerfacecolor="gray",
           markeredgecolor="white", markersize=10, label=BENCH_LABEL[b])
    for b in ["EP", "LU"]
]
color_legend = [
    Patch(facecolor=COLOR_REPLACE, label="REPLACE_RESUME faster (measured)"),
    Patch(facecolor=COLOR_DEGRADED, label="DEGRADED_RESUME faster (measured)"),
]

leg1 = ax.legend(handles=shape_legend, title="Benchmark (shape)",
                  loc="upper right", bbox_to_anchor=(0.99, 0.98),
                  fontsize=8.5, title_fontsize=8.5, framealpha=0.95)
ax.add_artist(leg1)
ax.legend(handles=color_legend, title="Measured winner (color)",
          loc="upper right", bbox_to_anchor=(0.99, 0.80),
          fontsize=8.5, title_fontsize=8.5, framealpha=0.95)

fig.subplots_adjust(top=0.80, right=0.98, left=0.08, bottom=0.12)
fig.savefig(OUT_DIR / "fig_adaptive_boundary.png", dpi=200)
print(f"Saved to {OUT_DIR / 'fig_adaptive_boundary.png'}")
print(f"Boundary at remaining_secs = {BOUNDARY:.1f}s")
