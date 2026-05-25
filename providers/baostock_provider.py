"""Baostock fallback adapter for A-share provider repair.

The provider bridge calls this only after the primary A-share adapter and the
Tushare path fail.  Missing dependencies are returned as explicit provider
failures; no mock financials are produced.
"""

from __future__ import annotations

import importlib.util
from typing import Any


def fetch(ticker: str) -> dict[str, Any]:
    if importlib.util.find_spec("baostock") is None:
        raise RuntimeError("baostock package is not installed")
    raise RuntimeError("baostock adapter is available but not enabled for v5 minimum payload repair")
