#!/usr/bin/env python3
"""Tiny dependency-free Markdown-to-basic-PDF exporter.

This is intentionally conservative: it writes a readable text-first PDF when
no full HTML/PDF engine is available. It does not pretend to be a high-fidelity
layout engine.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path


def _clean(md: str) -> list[str]:
    lines: list[str] = []
    for raw in md.splitlines():
        line = raw.strip()
        if not line:
            lines.append("")
            continue
        line = re.sub(r"!\[[^\]]*\]\([^)]+\)", "[chart image referenced in Markdown report]", line)
        line = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", line)
        line = line.replace("|", "  ")
        line = re.sub(r"[*_`>#]", "", line)
        line = re.sub(r"\s+", " ", line).strip()
        if line:
            lines.append(line[:115])
    return lines


def _pdf_escape(text: str) -> str:
    return text.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")


def write_basic_pdf(lines: list[str], out: Path) -> None:
    pages: list[list[str]] = []
    page: list[str] = []
    for line in lines:
        page.append(line)
        if len(page) >= 44:
            pages.append(page)
            page = []
    if page:
        pages.append(page)
    if not pages:
        pages = [["Empty report"]]

    objects: list[str] = []
    objects.append("<< /Type /Catalog /Pages 2 0 R >>")
    kids = " ".join(f"{3 + i * 2} 0 R" for i in range(len(pages)))
    objects.append(f"<< /Type /Pages /Kids [{kids}] /Count {len(pages)} >>")

    for idx, page_lines in enumerate(pages):
        page_obj = 3 + idx * 2
        content_obj = page_obj + 1
        objects.append(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> /Contents {content_obj} 0 R >>")
        y = 760
        stream_lines = ["BT", "/F1 10 Tf"]
        for line in page_lines:
            if not line:
                y -= 10
                continue
            stream_lines.append(f"50 {y} Td ({_pdf_escape(line)}) Tj")
            stream_lines.append(f"-50 -14 Td")
            y -= 14
        stream_lines.append("ET")
        stream = "\n".join(stream_lines)
        objects.append(f"<< /Length {len(stream.encode('latin-1', errors='replace'))} >>\nstream\n{stream}\nendstream")

    chunks = ["%PDF-1.4\n%\xE2\xE3\xCF\xD3\n"]
    offsets = [0]
    for i, obj in enumerate(objects, start=1):
        offsets.append(sum(len(c.encode("latin-1", errors="replace")) for c in chunks))
        chunks.append(f"{i} 0 obj\n{obj}\nendobj\n")
    xref = sum(len(c.encode("latin-1", errors="replace")) for c in chunks)
    chunks.append(f"xref\n0 {len(objects)+1}\n0000000000 65535 f \n")
    for off in offsets[1:]:
        chunks.append(f"{off:010d} 00000 n \n")
    chunks.append(f"trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes("".join(chunks).encode("latin-1", errors="replace"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--markdown", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    md = Path(args.markdown).read_text(encoding="utf-8", errors="ignore")
    lines = _clean(md)
    write_basic_pdf(lines, Path(args.out))


if __name__ == "__main__":
    main()

