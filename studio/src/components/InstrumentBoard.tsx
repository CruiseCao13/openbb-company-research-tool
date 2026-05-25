import { useTranslation } from "react-i18next";
import type { RunDetail } from "../types/app";

type BadgeVariant =
  | "PASS"
  | "WARNING"
  | "FAIL"
  | "DATA_GAP"
  | "EXTERNAL_AI"
  | "LOCAL_MOCK"
  | "HUMAN_REVIEW"
  | "PROVIDER_MOCK"
  | "CACHE"
  | "UNKNOWN";

function statusToBadge(status: string | null): BadgeVariant {
  const normalized = status?.toUpperCase() ?? "UNKNOWN";
  if (normalized.includes("PASS")) return "PASS";
  if (normalized.includes("FAIL")) return "FAIL";
  if (normalized.includes("WARN") || normalized.includes("REVIEW")) return "WARNING";
  if (normalized.includes("GAP")) return "DATA_GAP";
  return "UNKNOWN";
}

function collectWarnings(detail: RunDetail | null): string[] {
  if (!detail) {
    return [];
  }

  const warnings = new Set<string>(detail.warnings);
  if (detail.status.human_review_required || detail.self_review.human_review_required) {
    warnings.add("Human review required");
  }
  if (detail.ai_usage.local_mock_used) {
    warnings.add("Local/mock AI output is not external AI proof");
  }
  if (detail.provider.mock) {
    warnings.add("Provider metadata says mock data was used");
  }
  for (const field of detail.provider.missing_fields) {
    warnings.add(`Missing provider field: ${field}`);
  }
  for (const gap of detail.blueprint.data_gaps) {
    warnings.add(`Data gap: ${gap}`);
  }
  return Array.from(warnings);
}

function StatusBadge({ variant }: { variant: BadgeVariant }): JSX.Element {
  return <span className={`status-badge status-badge--${variant.toLowerCase()}`} data-tooltip={variant}>{variant}</span>;
}

function SimpleList({ emptyLabel, items, limit = 8 }: { emptyLabel: string; items: string[]; limit?: number }): JSX.Element {
  if (items.length === 0) {
    return <p className="muted-copy">{emptyLabel}</p>;
  }
  return (
    <ul className="compact-list">
      {items.slice(0, limit).map((item) => <li key={item}>{item}</li>)}
      {items.length > limit ? <li>{items.length - limit} more</li> : null}
    </ul>
  );
}

function gaugeVariant(status: BadgeVariant): "good" | "warning" | "risk" | "unknown" {
  if (status === "PASS" || status === "EXTERNAL_AI" || status === "CACHE") {
    return "good";
  }
  if (status === "WARNING" || status === "DATA_GAP" || status === "HUMAN_REVIEW" || status === "LOCAL_MOCK") {
    return "warning";
  }
  if (status === "FAIL" || status === "PROVIDER_MOCK") {
    return "risk";
  }
  return "unknown";
}

const labelAbbreviations = new Map<string, string>([
  ["Operating Cash Flow", "OCF"],
  ["Free Cash Flow", "FCF"],
  ["Capital Expenditure", "Capex"],
  ["Financial Report Framework Coverage", "Framework Coverage"],
  ["Money Flow Specificity", "Flow Specificity"],
  ["Provider Coverage", "Provider"],
  ["Human Review Required", "Human Review"],
  ["Human Review", "Human Review"],
  ["Template Leakage", "Template Leak"],
  ["Market Expectations", "Expectations"],
  ["Data Confidence", "Data Confidence"],
  ["Cash Flow Quality", "Cash Flow"],
  ["Valuation Risk", "Valuation Risk"],
  ["财报阅读框架覆盖情况", "框架覆盖"],
  ["资金流具体性", "资金流"],
  ["现金流具体度", "现金流"],
  ["是否需要人工复核", "人工复核"],
  ["人工复核", "人工复核"],
  ["数据来源覆盖度", "数据源"],
  ["数据源覆盖", "数据源"],
  ["估值风险", "估值风险"],
  ["现金流质量", "现金流"],
  ["数据可信度", "可信度"],
  ["模板泄漏", "模板"]
]);

function shortLabel(label: string): string {
  return labelAbbreviations.get(label) ?? label;
}

function GaugeDial({
  detail,
  label,
  status,
  summary
}: {
  detail: string;
  label: string;
  status: BadgeVariant;
  summary: string;
}): JSX.Element {
  const variant = gaugeVariant(status);
  return (
    <article
      aria-label={`${label}: ${summary}`}
      className={`gauge-dial gauge-dial--${variant}`}
      data-tooltip={`${label} / ${summary}. ${detail}`}
    >
      <div className="gauge-dial__meter" aria-hidden="true">
        <span className="gauge-value-line"><b>{variant === "unknown" ? "--" : status}</b></span>
      </div>
      <div className="gauge-dial__body">
        <strong className="gauge-label" data-full-label={label}>{shortLabel(label)}</strong>
        <span>{summary}</span>
        <p>{detail}</p>
      </div>
    </article>
  );
}

function strongestSignal(detail: RunDetail | null, warnings: string[]): BadgeVariant {
  if (!detail) {
    return "UNKNOWN";
  }
  if (detail.status.human_review_required || detail.self_review.human_review_required) {
    return "HUMAN_REVIEW";
  }
  if (detail.provider.mock) {
    return "PROVIDER_MOCK";
  }
  if ((detail.status.overall_status ?? "").toUpperCase().includes("FAIL")) {
    return "FAIL";
  }
  if (warnings.length > 0) {
    return "WARNING";
  }
  if (detail.provider.missing_fields.length > 0 || detail.blueprint.data_gaps.length > 0) {
    return "DATA_GAP";
  }
  return statusToBadge(detail.status.overall_status);
}

function insightHeadline(detail: RunDetail | null, warnings: string[]): string {
  if (!detail) {
    return "Load a run to turn raw artifacts into a visual research instrument.";
  }
  if (detail.status.human_review_required || detail.self_review.human_review_required) {
    return "Human review is the strongest signal; inspect warnings before relying on the run.";
  }
  if (warnings.length > 0) {
    return "The run is usable as a visual map, but warnings should guide the next check.";
  }
  if (detail.financial_interpretation.where_money_comes_from || detail.financial_interpretation.where_money_goes) {
    return "Money-flow mechanism is available; use the visual flow first, then verify artifacts.";
  }
  return "Research metadata is loaded, but money-flow specifics remain data-limited.";
}

function InsightDisclosure({
  badge,
  emptyLabel,
  items,
  title
}: {
  badge: BadgeVariant;
  emptyLabel: string;
  items: string[];
  title: string;
}): JSX.Element {
  return (
    <details className="insight-disclosure">
      <summary>
        <span className="insight-disclosure__title">{title}</span>
        <span className="insight-disclosure__meta">
          <StatusBadge variant={badge} />
          <b>{items.length}</b>
        </span>
      </summary>
      <SimpleList emptyLabel={emptyLabel} items={items} limit={5} />
    </details>
  );
}

export function InstrumentBoard({ detail, warningsFirst }: { detail: RunDetail | null; warningsFirst: boolean }): JSX.Element {
  const { t } = useTranslation();
  const warnings = collectWarnings(detail);
  const gaps = detail ? [...detail.provider.missing_fields, ...detail.blueprint.data_gaps] : [];
  const nextChecks = detail?.blueprint.next_checks ?? [];
  const frameworkChecks = detail
    ? [
        detail.self_review.framework_fit_check ? `Framework: ${detail.self_review.framework_fit_check}` : null,
        detail.self_review.numeric_consistency_check ? `Numbers: ${detail.self_review.numeric_consistency_check}` : null,
        detail.self_review.money_flow_check ? `Money flow: ${detail.self_review.money_flow_check}` : null,
        detail.self_review.company_understanding_check ? `Company: ${detail.self_review.company_understanding_check}` : null
      ].filter((item): item is string => item !== null)
    : [];
  const blueprintItems = detail
    ? [
        detail.blueprint.core_thesis ? `Core: ${detail.blueprint.core_thesis}` : null,
        ...detail.blueprint.must_analyze.map((item) => `Analyze: ${item}`),
        ...detail.blueprint.red_flags.map((item) => `Red flag: ${item}`)
      ].filter((item): item is string => item !== null)
    : [];
  const signal = strongestSignal(detail, warnings);
  const gaugeItems = [
    {
      label: t("dataConfidenceGauge"),
      status: detail ? statusToBadge(detail.status.overall_status) : "UNKNOWN",
      summary: detail?.status.overall_status ?? t("unknown"),
      detail: detail ? t("statusReadFromMetadata") : t("selectRun")
    },
    {
      label: t("cashFlowQualityGauge"),
      status: detail?.financial_interpretation.cash_flow_explanation ? "PASS" : "DATA_GAP",
      summary: detail?.financial_interpretation.cash_flow_explanation ? t("available") : t("dataGap"),
      detail: detail?.financial_interpretation.cash_flow_explanation ?? t("cashFlowExplanationMissing")
    },
    {
      label: t("moneyFlowSpecificityGauge"),
      status: detail?.financial_interpretation.where_money_comes_from && detail.financial_interpretation.where_money_goes ? "PASS" : "DATA_GAP",
      summary: detail?.financial_interpretation.where_money_comes_from ? t("available") : t("dataGap"),
      detail: detail?.financial_interpretation.where_money_comes_from ?? t("moneySourceMissing")
    },
    {
      label: t("humanReviewGauge"),
      status: detail?.status.human_review_required || detail?.self_review.human_review_required ? "HUMAN_REVIEW" : detail ? "PASS" : "UNKNOWN",
      summary: detail?.status.human_review_required || detail?.self_review.human_review_required ? t("required") : detail ? "OK" : t("unknown"),
      detail: detail ? t("humanReviewReadFromMetadata") : t("selectRun")
    },
    {
      label: t("providerCoverageGauge"),
      status: detail?.provider.mock ? "PROVIDER_MOCK" : detail?.provider.missing_fields.length ? "WARNING" : detail ? "PASS" : "UNKNOWN",
      summary: detail?.provider.provider ?? t("unknown"),
      detail: detail?.provider.missing_fields.join("; ") || t("providerCoverageComplete")
    },
    {
      label: t("valuationRiskGauge"),
      status: detail?.financial_interpretation.valuation_method_fit ? "WARNING" : "UNKNOWN",
      summary: detail?.financial_interpretation.valuation_method_fit ? t("available") : t("unknown"),
      detail: detail?.financial_interpretation.valuation_method_fit ?? t("valuationFitMissing")
    },
    {
      label: t("templateLeakageGauge"),
      status: detail ? "UNKNOWN" : "UNKNOWN",
      summary: t("unknown"),
      detail: t("templateLeakageMissing")
    }
  ] satisfies Array<{ detail: string; label: string; status: BadgeVariant; summary: string }>;

  const gaugePanel = (
    <section className="gauge-dashboard" key="gauges" aria-label="Gauge dashboard">
      <div className="gauge-dashboard__header">
        <span className="card-label">{t("gaugeDashboard")}</span>
        <small>{t("hoverForMeaning")}</small>
      </div>
      <div className="gauge-dashboard__grid">
        {gaugeItems.map((item) => <GaugeDial key={item.label} {...item} />)}
      </div>
    </section>
  );
  const funnelPanel = (
    <section className="insight-card insight-card--funnel" key="funnel">
      <div className="insight-verdict">
        <span className="card-label">{t("strongestSignal")}</span>
        <StatusBadge variant={signal} />
        <strong>{insightHeadline(detail, warnings)}</strong>
      </div>
      <div className="insight-pill-row" aria-label="Insight summary pills">
        <span>{warnings.length} {t("warnings")}</span>
        <span>{gaps.length} {t("dataGaps")}</span>
        <span>{nextChecks.length} {t("nextQuestions")}</span>
      </div>
      <div className="insight-disclosure-stack">
        <InsightDisclosure badge={warnings.length > 0 ? "WARNING" : "PASS"} emptyLabel={t("noWarnings")} items={warnings} title={t("warnings")} />
        <InsightDisclosure badge={gaps.length > 0 ? "DATA_GAP" : "PASS"} emptyLabel={t("providerCoverageComplete")} items={gaps} title={t("dataGaps")} />
        <InsightDisclosure badge={nextChecks.length > 0 ? "UNKNOWN" : "DATA_GAP"} emptyLabel={t("noNextChecks")} items={nextChecks} title={t("nextQuestions")} />
        <InsightDisclosure badge={frameworkChecks.length > 0 ? "UNKNOWN" : "DATA_GAP"} emptyLabel={t("templateLeakageMissing")} items={frameworkChecks} title={t("frameworkCoverage")} />
        <InsightDisclosure badge={blueprintItems.length > 0 ? "UNKNOWN" : "DATA_GAP"} emptyLabel={t("noNextChecks")} items={blueprintItems} title={t("blueprint")} />
      </div>
    </section>
  );

  return (
    <aside className="insight-rail" aria-label="Run insights">
      {warningsFirst ? [funnelPanel, gaugePanel] : [gaugePanel, funnelPanel]}
    </aside>
  );
}
