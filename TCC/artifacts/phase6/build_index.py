#!/usr/bin/env python3
"""
build_index.py — Extract PDFs to TXT and generate REFERENCES_INDEX.md.

Directory layout (all relative to this script):
    pdfs/<theme>/*.pdf          source PDFs, organized by theme folder
    references/<theme>/*.txt    extracted full text (mirrors pdfs/ structure)
    REFERENCES_INDEX.md         generated index, organized by theme

Usage:
    python build_index.py               # extract new PDFs only, rebuild index
    python build_index.py --force       # re-extract ALL PDFs, rebuild index

Workflow:
    Add paper    — drop PDF in the correct pdfs/<theme>/ folder, re-run
    Move paper   — move PDF (and its .txt in references/) to new theme, re-run
    Rename theme — rename the pdfs/<theme>/ folder, re-run
"""

import argparse
import re
import sys
import textwrap
from datetime import date
from pathlib import Path

try:
    import fitz  # pymupdf
except ImportError:
    sys.exit("pymupdf not found. Run: pip install pymupdf")

BASE_DIR    = Path(__file__).parent
PDFS_DIR    = BASE_DIR / "pdfs"
REFS_DIR    = BASE_DIR / "references"
INDEX_FILE  = BASE_DIR / "REFERENCES_INDEX.md"

IGNORED_DIRS  = {".git", "__pycache__"}
SUMMARY_SENTS = 5   # sentences kept per paper in the index


# ---------------------------------------------------------------------------
# PDF extraction
# ---------------------------------------------------------------------------

def extract_text(pdf_path: Path) -> str:
    doc = fitz.open(str(pdf_path))
    pages = []
    for page in doc:
        text = page.get_text("text")
        if text.strip():
            pages.append(text)
    doc.close()
    return "\n\n".join(pages)


def extract_summary(text: str, n: int = SUMMARY_SENTS) -> str:
    """Return up to n sentences from the abstract, or from the first prose block."""
    match = re.search(
        r'\bAbstract\b[:\s\-–—]*(.*?)(?:\n{2,}|\Z)',
        text, re.IGNORECASE | re.DOTALL
    )
    if match:
        candidate = match.group(1).strip()
        if len(candidate) > 100:
            return " ".join(re.split(r'(?<=[.!?])\s+', candidate)[:n]).strip()

    # Fall back: skip short lines (titles/headers) and take first prose
    lines = [l.strip() for l in text.splitlines() if len(l.strip()) > 60]
    prose = " ".join(lines[:40])
    return " ".join(re.split(r'(?<=[.!?])\s+', prose)[:n]).strip()


# ---------------------------------------------------------------------------
# Theme/paper discovery
# ---------------------------------------------------------------------------

def theme_label(folder_name: str) -> str:
    return folder_name.replace("-", " ").replace("_", " ").title()


def collect(force: bool) -> dict[str, list[dict]]:
    """
    Walk pdfs/<theme>/ folders, extract PDFs as needed, return structured data.
    Root-level PDFs in pdfs/ are placed under a synthetic "project-documents" theme.
    { theme_folder: [ {title, txt_path, summary}, ... ] }
    """
    themes: dict[str, list[dict]] = {}

    # Handle PDFs sitting directly in pdfs/ root (not inside a theme subfolder)
    root_papers = []
    for pdf in sorted(PDFS_DIR.glob("*.pdf")):
        out_dir = REFS_DIR / "project-documents"
        out_dir.mkdir(parents=True, exist_ok=True)
        txt_path = out_dir / (pdf.stem + ".txt")
        if force or not txt_path.exists():
            print(f"  extracting  {pdf.name}")
            try:
                text = extract_text(pdf)
                txt_path.write_text(text, encoding="utf-8")
            except Exception as exc:
                print(f"  WARNING: {pdf.name}: {exc}")
                continue
        else:
            text = txt_path.read_text(encoding="utf-8", errors="replace")
        title   = re.sub(r'[-_]+', ' ', pdf.stem).strip()
        summary = extract_summary(text)
        root_papers.append({
            "title":    title,
            "txt_path": txt_path.relative_to(BASE_DIR),
            "summary":  summary,
        })
    if root_papers:
        themes["project-documents"] = root_papers

    for theme_dir in sorted(PDFS_DIR.iterdir()):
        if not theme_dir.is_dir() or theme_dir.name in IGNORED_DIRS:
            continue

        out_dir = REFS_DIR / theme_dir.name
        papers  = []

        for pdf in sorted(theme_dir.glob("*.pdf")):
            out_dir.mkdir(parents=True, exist_ok=True)
            txt_path = out_dir / (pdf.stem + ".txt")

            if force or not txt_path.exists():
                print(f"  extracting  {theme_dir.name}/{pdf.name}")
                try:
                    text = extract_text(pdf)
                    txt_path.write_text(text, encoding="utf-8")
                except Exception as exc:
                    print(f"  WARNING: {pdf.name}: {exc}")
                    continue
            else:
                text = txt_path.read_text(encoding="utf-8", errors="replace")

            title   = re.sub(r'[-_]+', ' ', pdf.stem).strip()
            summary = extract_summary(text)
            papers.append({
                "title":    title,
                "txt_path": txt_path.relative_to(BASE_DIR),
                "summary":  summary,
            })

        if papers:
            themes[theme_dir.name] = papers

    return themes


# ---------------------------------------------------------------------------
# Index rendering
# ---------------------------------------------------------------------------

def render_index(themes: dict[str, list[dict]]) -> str:
    total  = sum(len(v) for v in themes.values())
    today  = date.today().isoformat()

    lines = [
        "# References Index",
        "",
        f"Generated: {today} — {total} papers across {len(themes)} themes.",
        "Re-run `build_index.py` after adding or moving papers.",
        "",
        "---",
        "",
        "## Table of Contents",
        "",
    ]

    for folder in sorted(themes):
        label = theme_label(folder)
        anchor = label.lower().replace(" ", "-")
        count  = len(themes[folder])
        lines.append(f"- [{label}](#{anchor}) ({count} paper{'s' if count > 1 else ''})")

    lines += ["", "---", ""]

    for folder in sorted(themes):
        label  = theme_label(folder)
        papers = themes[folder]

        lines += [f"## {label}", ""]

        for p in papers:
            lines += [
                f"### {p['title']}",
                f"**Full text:** [{p['txt_path'].name}]({p['txt_path']})",
                "",
            ]
            if p["summary"]:
                lines.append(textwrap.fill(p["summary"], width=100))
            else:
                lines.append("_Summary unavailable — check the full text._")
            lines += ["", "---", ""]

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--force", action="store_true",
                        help="Re-extract all PDFs even if .txt already exists")
    args = parser.parse_args()

    if not PDFS_DIR.exists():
        sys.exit(f"PDFs directory not found: {PDFS_DIR}")

    print(f"Source : {PDFS_DIR}")
    print(f"Output : {REFS_DIR}")
    print(f"Mode   : {'force re-extract' if args.force else 'incremental'}\n")

    themes = collect(force=args.force)

    total = sum(len(v) for v in themes.values())
    print(f"\nIndexing {total} papers across {len(themes)} themes...")

    INDEX_FILE.write_text(render_index(themes), encoding="utf-8")
    print(f"Index  : {INDEX_FILE.relative_to(BASE_DIR.parent.parent)}")


if __name__ == "__main__":
    main()
