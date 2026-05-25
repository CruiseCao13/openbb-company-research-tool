type LogoMarkStatus = "PASS" | "WARNING" | "FAIL" | "DATA_GAP" | "UNKNOWN" | string | null | undefined;

function statusToBadge(status: LogoMarkStatus): "pass" | "warning" | "fail" | "data_gap" | "unknown" {
  const normalized = status?.toUpperCase() ?? "UNKNOWN";
  if (normalized.includes("PASS")) return "pass";
  if (normalized.includes("FAIL")) return "fail";
  if (normalized.includes("WARN") || normalized.includes("REVIEW")) return "warning";
  if (normalized.includes("GAP")) return "data_gap";
  return "unknown";
}

function tickerMonogram(ticker: string | null | undefined): string {
  if (!ticker) {
    return "RS";
  }
  return ticker
    .replace(/\.(SH|SZ)$/u, "")
    .split(/[^A-Z0-9]+/u)
    .filter(Boolean)
    .join("")
    .slice(0, 3)
    .toUpperCase();
}

export function LogoMark({
  market,
  status,
  ticker
}: {
  market: string | null | undefined;
  status: LogoMarkStatus;
  ticker: string | null | undefined;
}): JSX.Element {
  return (
    <div className={`company-monogram company-monogram--${statusToBadge(status)}`} aria-hidden="true">
      <strong>{tickerMonogram(ticker)}</strong>
      <span>{market ?? "MKT"}</span>
    </div>
  );
}
