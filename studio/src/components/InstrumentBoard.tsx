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
  return <span className={`status-badge status-badge--${variant.toLowerCase()}`}>{variant}</span>;
}

function SimpleList({ emptyLabel, items }: { emptyLabel: string; items: string[] }): JSX.Element {
  if (items.length === 0) {
    return <p className="muted-copy">{emptyLabel}</p>;
  }
  return (
    <ul className="compact-list">
      {items.slice(0, 8).map((item) => <li key={item}>{item}</li>)}
      {items.length > 8 ? <li>{items.length - 8} more</li> : null}
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
    <article className={`gauge-dial gauge-dial--${variant}`}>
      <div className="gauge-dial__meter" aria-hidden="true">
        <span>{variant === "unknown" ? "--" : status}</span>
      </div>
      <div className="gauge-dial__body">
        <strong>{label}</strong>
        <span>{summary}</span>
        <p>{detail}</p>
      </div>
    </article>
  );
}

export function InstrumentBoard({ detail, warningsFirst }: { detail: RunDetail | null; warningsFirst: boolean }): JSX.Element {
  const { t } = useTranslation();
  const warnings = collectWarnings(detail);
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

  const warningCard = (
    <section className="insight-card insight-card--warning" key="warnings">
      <div className="card-header">
        <span className="card-label">{t("warnings")}</span>
        <StatusBadge variant={warnings.length > 0 ? "WARNING" : "PASS"} />
      </div>
      <SimpleList emptyLabel={t("noWarnings")} items={warnings} />
    </section>
  );
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
  const questionsCard = (
    <section className="insight-card" key="questions">
      <span className="card-label">{t("nextQuestions")}</span>
      <SimpleList emptyLabel={t("noNextChecks")} items={detail?.blueprint.next_checks ?? []} />
    </section>
  );

  return (
    <aside className="insight-rail" aria-label="Run insights">
      {warningsFirst ? [warningCard, gaugePanel, questionsCard] : [gaugePanel, warningCard, questionsCard]}
    </aside>
  );
}
