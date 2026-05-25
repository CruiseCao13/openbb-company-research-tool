"""Tushare fallback adapter for A-share provider repair.

This adapter is intentionally conservative.  It only reports availability when
both a TUSHARE_TOKEN and the tushare package are present; provider_common.py
then falls through to the next real provider instead of fabricating data.
"""

from __future__ import annotations

import importlib.util
import os
from typing import Any


def fetch(ticker: str) -> dict[str, Any]:
    token = os.environ.get("TUSHARE_TOKEN")
    if not token:
        raise RuntimeError("TUSHARE_TOKEN is not set")
    if importlib.util.find_spec("tushare") is None:
        raise RuntimeError("tushare package is not installed")
    raise RuntimeError("tushare adapter is available but not enabled for v5 minimum payload repair")
