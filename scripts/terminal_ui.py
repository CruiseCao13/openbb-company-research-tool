"""Terminal rendering helpers for the research CLI."""

from __future__ import annotations

from contextlib import nullcontext
from pathlib import Path
from typing import Any


class TerminalUI:
    def __init__(self, enabled: bool = True) -> None:
        self.enabled = enabled
        self._console = None
        self._rich = False
        if enabled:
            try:
                from rich.console import Console

                self._console = Console()
                self._rich = True
            except Exception:
                self._console = None
                self._rich = False

    def print(self, message: str = "") -> None:
        if self._rich and self._console is not None:
            self._console.print(message)
        else:
            print(_strip_rich(message))

    def status(self, message: str):
        if self._rich and self._console is not None:
            return self._console.status(message, spinner="dots")
        print(_strip_rich(message))
        return nullcontext()


_UI = TerminalUI(enabled=True)


def configure(enabled: bool = True) -> None:
    global _UI
    _UI = TerminalUI(enabled=enabled)


def _strip_rich(message: str) -> str:
    for token in [
        "[bold]",
        "[/bold]",
        "[cyan]",
        "[/cyan]",
        "[green]",
        "[/green]",
        "[yellow]",
        "[/yellow]",
        "[red]",
        "[/red]",
        "[dim]",
        "[/dim]",
    ]:
        message = message.replace(token, "")
    return message


def print_app_banner(version: str) -> None:
    if _UI._rich:
        _UI.print(f"\n[bold]OpenBB Company Research Tool v{version}[/bold]")
        _UI.print("[dim]Data-first research workflow with optional AI review[/dim]\n")
    else:
        _UI.print(f"\nOpenBB Company Research Tool v{version}")
        _UI.print("Data-first research workflow with optional AI review\n")


def print_run_config(
    ticker: str,
    benchmark: str,
    ai_review: bool,
    archive_enabled: bool,
    model: str | None = None,
) -> None:
    mode = "Research + AI Review" if ai_review else "Research"
    _UI.print(f"Target: {ticker}")
    _UI.print(f"Benchmark: {benchmark}")
    _UI.print(f"Mode: {mode}")
    if ai_review and model:
        _UI.print(f"Model: {model}")
    _UI.print(f"Archive: {'enabled' if archive_enabled else 'disabled'}\n")


def step_start(label: str) -> None:
    _UI.print(f"{label:<42} ...")


def step_done(label: str, detail: str | None = None) -> None:
    suffix = f"  {detail}" if detail else ""
    _UI.print(f"[green]{label:<42} done[/green]{suffix}")


def step_warn(label: str, detail: str | None = None) -> None:
    suffix = f"  {detail}" if detail else ""
    _UI.print(f"[yellow]{label:<42} warning[/yellow]{suffix}")


def step_error(label: str, detail: str | None = None) -> None:
    suffix = f"  {detail}" if detail else ""
    _UI.print(f"[red]{label:<42} error[/red]{suffix}")


def print_ai_review_status(status: str, model: str | None = None, error: str | None = None) -> None:
    detail = f"model={model}" if model else None
    if status == "completed":
        step_done("[7/8] Running AI review", detail)
    elif status == "skipped":
        step_warn("[7/8] Running AI review", error or detail)
    elif status == "disabled":
        step_warn("[7/8] Running AI review", "disabled")
    else:
        step_error("[7/8] Running AI review", error or status)


def ai_review_spinner():
    return _UI.status("Calling OpenAI API, this may take a few seconds...")


def print_final_outputs(report_path: str | Path, dashboard_path: str | Path, run_dir: str | Path) -> None:
    _UI.print("\n[bold]Outputs[/bold]")
    _UI.print(f"Report: {report_path}")
    _UI.print(f"Dashboard: {dashboard_path}")
    _UI.print(f"Run folder: {run_dir}")


def plain_mode_does_not_crash() -> bool:
    ui = TerminalUI(enabled=False)
    ui.print("terminal-ui-ok")
    return True

