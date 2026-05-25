import { type ReactNode, useState } from "react";
import { sankey, sankeyLinkHorizontal, type SankeyGraph } from "d3-sankey";
import { useTranslation } from "react-i18next";
import { artifactImageSrc, openArtifact, revealInFolder } from "../lib/tauri";
import type { RunDetail, RunDetailStatus } from "../types/app";

export type RunDetailTab = "summary" | "charts" | "audit" | "gaps" | "artifacts";

type DetailBadgeVariant = "PASS" | "WARNING" | "FAIL" | "DATA_GAP" | "EXTERNAL_AI" | "LOCAL_MOCK" | "UNKNOWN";
type AuditBadgeVariant = DetailBadgeVariant | "CACHED" | "SKIPPED";

type RunDetailPanelProps = {
  activeTab: RunDetailTab;
  detail: RunDetail | null;
  error: string | null;
  status: RunDetailStatus;
};

function StatusBadge({ variant }: { variant: DetailBadgeVariant }): JSX.Element {
  return <span className={`status-badge status-badge--${variant.toLowerCase()}`}>{variant}</span>;
}

function AuditStatusBadge({ status }: { status: string }): JSX.Element {
  const normalized = status.toUpperCase();
  const variant: AuditBadgeVariant =
    normalized === "PASS" ||
    normalized === "WARNING" ||
    normalized === "FAIL" ||
    normalized === "CACHED" ||
    normalized === "SKIPPED"
      ? (normalized as AuditBadgeVariant)
      : "UNKNOWN";
  return <span className={`status-badge status-badge--${variant.toLowerCase()}`}>{variant}</span>;
}

function statusToBadge(status: string | null): DetailBadgeVariant {
  const normalized = status?.toUpperCase() ?? "UNKNOWN";
  if (normalized.includes("PASS")) {
    return "PASS";
  }
  if (normalized.includes("FAIL")) {
    return "FAIL";
  }
  if (normalized.includes("WARN") || normalized.includes("REVIEW")) {
    return "WARNING";
  }
  if (normalized.includes("GAP")) {
    return "DATA_GAP";
  }
  return "UNKNOWN";
}

function booleanLabel(value: boolean | null): string {
  if (value === null) {
    return "unknown";
  }
  return value ? "yes" : "no";
}

function DetailSection({
  badge,
  children,
  title
}: {
  badge: DetailBadgeVariant;
  children: ReactNode;
  title: string;
}): JSX.Element {
  return (
    <section className="run-detail-card" aria-label={`${title} card`}>
      <div className="card-header">
        <span className="card-label">{title}</span>
        <StatusBadge variant={badge} />
      </div>
      {children}
    </section>
  );
}

type SankeyNodeDatum = {
  name: string;
  kind: "inflow" | "engine" | "reinvestment" | "liquidity" | "gap" | "return";
};

type SankeyLinkDatum = {
  source: number;
  target: number;
  value: number;
  label: string;
  kind: "inflow" | "cash" | "reinvestment" | "risk" | "gap";
};

function compactText(value: string | null | undefined, fallback: string): string {
  const text = value?.trim();
  if (!text) {
    return fallback;
  }
  return text.length > 92 ? `${text.slice(0, 89)}...` : text;
}

function buildMoneyFlowGraph(detail: RunDetail): SankeyGraph<SankeyNodeDatum, SankeyLinkDatum> | null {
  const sourceText = detail.financial_interpretation.where_money_comes_from;
  const useText = detail.financial_interpretation.where_money_goes;
  const cashText = detail.financial_interpretation.cash_flow_explanation;
  const debtText = detail.financial_interpretation.debt_and_financing;
  const hasMechanism = Boolean(sourceText || useText || cashText || debtText);

  if (!hasMechanism) {
    return null;
  }

  return {
    nodes: [
      { name: "Revenue", kind: "inflow" },
      { name: "Cash engine", kind: "engine" },
      { name: "Reinvestment", kind: "reinvestment" },
      { name: "Liquidity", kind: "liquidity" },
      { name: "Data gaps", kind: "gap" },
      { name: "Shareholder return", kind: "return" }
    ],
    links: [
      {
        source: 0,
        target: 1,
        value: 1,
        label: compactText(sourceText, "Revenue or operating source not specified."),
        kind: "inflow"
      },
      {
        source: 1,
        target: 2,
        value: 1,
        label: compactText(useText, "Cash uses not specified."),
        kind: "reinvestment"
      },
      {
        source: 1,
        target: 3,
        value: debtText ? 1 : 0.45,
        label: compactText(debtText, "Financing pressure not specified."),
        kind: debtText ? "risk" : "cash"
      },
      {
        source: 1,
        target: 4,
        value: detail.blueprint.data_gaps.length || detail.provider.missing_fields.length ? 0.75 : 0.25,
        label: compactText([...detail.blueprint.data_gaps, ...detail.provider.missing_fields].join("; "), "No explicit data gap reported."),
        kind: "gap"
      },
      {
        source: 3,
        target: 5,
        value: detail.financial_interpretation.cash_flow_explanation ? 0.55 : 0.25,
        label: compactText(detail.financial_interpretation.cash_flow_explanation, "Shareholder return support is not specified."),
        kind: "cash"
      }
    ]
  };
}

export function MoneyFlowSankey({ detail }: { detail: RunDetail }): JSX.Element {
  const { t } = useTranslation();
  const graph = buildMoneyFlowGraph(detail);

  if (!graph) {
    return (
      <DetailSection badge="DATA_GAP" title="Money Flow Sankey">
        <div className="sankey-empty">
          <strong>{t("moneyFlow")}</strong>
          <span>Money-flow mechanism fields are not available for this run.</span>
        </div>
      </DetailSection>
    );
  }

  const width = 980;
  const height = 540;
  const layout = sankey<SankeyNodeDatum, SankeyLinkDatum>()
    .nodeWidth(22)
    .nodePadding(34)
    .extent([
      [16, 18],
      [width - 18, height - 18]
    ]);
  const rendered = layout({
    nodes: graph.nodes.map((node) => ({ ...node })),
    links: graph.links.filter((link) => link.value > 0).map((link) => ({ ...link }))
  });
  const path = sankeyLinkHorizontal<SankeyNodeDatum, SankeyLinkDatum>();

  return (
    <DetailSection badge="UNKNOWN" title={t("vascularMoneyFlow")}>
      <div className="sankey-panel sankey-panel--vascular">
        <div className="sankey-mode-badge">
          <span>Qualitative flow map</span>
          <small>Not amount-scaled</small>
        </div>
        <svg className="sankey-svg" viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Qualitative money flow Sankey">
          <defs>
            <filter id="sankey-vascular-glow" x="-30%" y="-30%" width="160%" height="160%">
              <feGaussianBlur stdDeviation="6" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
            <linearGradient id="sankey-link-gradient-inflow" x1="0%" x2="100%" y1="0%" y2="0%">
              <stop offset="0%" stopColor="rgba(52, 211, 153, 0.22)" />
              <stop offset="52%" stopColor="rgba(52, 211, 153, 0.9)" />
              <stop offset="100%" stopColor="rgba(143, 211, 255, 0.62)" />
            </linearGradient>
            <linearGradient id="sankey-link-gradient-reinvestment" x1="0%" x2="100%" y1="0%" y2="0%">
              <stop offset="0%" stopColor="rgba(143, 211, 255, 0.2)" />
              <stop offset="58%" stopColor="rgba(96, 165, 250, 0.82)" />
              <stop offset="100%" stopColor="rgba(129, 140, 248, 0.42)" />
            </linearGradient>
            <linearGradient id="sankey-link-gradient-risk" x1="0%" x2="100%" y1="0%" y2="0%">
              <stop offset="0%" stopColor="rgba(251, 146, 60, 0.25)" />
              <stop offset="58%" stopColor="rgba(251, 146, 60, 0.85)" />
              <stop offset="100%" stopColor="rgba(244, 63, 94, 0.48)" />
            </linearGradient>
            <linearGradient id="sankey-link-gradient-gap" x1="0%" x2="100%" y1="0%" y2="0%">
              <stop offset="0%" stopColor="rgba(148, 163, 184, 0.14)" />
              <stop offset="70%" stopColor="rgba(251, 146, 60, 0.48)" />
            </linearGradient>
          </defs>
          {rendered.links.map((link, index) => (
            <path
              className={`sankey-link sankey-link--${link.kind}`}
              d={path(link) ?? undefined}
              key={`${link.index ?? index}-${link.label}`}
              strokeWidth={Math.max(18, (link.width ?? 8) * 1.4)}
            >
              <title>{`${link.label}\n${t("qualitativeSankey")}`}</title>
            </path>
          ))}
          {rendered.nodes.map((node) => (
            <g className={`sankey-node sankey-node--${node.kind}`} key={node.name}>
              <rect height={(node.y1 ?? 0) - (node.y0 ?? 0)} rx="13" width={(node.x1 ?? 0) - (node.x0 ?? 0)} x={node.x0} y={node.y0} />
              <text
                textAnchor={(node.x0 ?? 0) < width / 2 ? "start" : "end"}
                x={(node.x0 ?? 0) < width / 2 ? (node.x1 ?? 0) + 10 : (node.x0 ?? 0) - 10}
                y={((node.y0 ?? 0) + (node.y1 ?? 0)) / 2}
              >
                {node.name}
              </text>
            </g>
          ))}
        </svg>
        <div className="sankey-sidecar">
          <span className="subsection-title">{t("limitation")}</span>
          <p>{t("qualitativeSankey")}</p>
          <span className="subsection-title">{t("source")}</span>
          <p>metadata/financial_interpretation.json + metadata/blueprint/data gaps loaded through Tauri IPC.</p>
          <span className="subsection-title">{t("nextCheck")}</span>
          <p>{detail.blueprint.next_checks[0] ?? "Open the report and validator audit before using this view for review."}</p>
        </div>
      </div>
    </DetailSection>
  );
}

function KeyValueRow({ label, value }: { label: string; value: ReactNode }): JSX.Element {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function BulletList({ emptyLabel, items, limit = 7 }: { emptyLabel: string; items: string[]; limit?: number }): JSX.Element {
  if (items.length === 0) {
    return <p className="muted-copy">{emptyLabel}</p>;
  }

  return (
    <ul className="compact-list">
      {items.slice(0, limit).map((item) => (
        <li key={item}>{item}</li>
      ))}
      {items.length > limit ? <li>{items.length - limit} more</li> : null}
    </ul>
  );
}

function EmptyRunDetailState({ detail, title }: { detail: string; title: string }): JSX.Element {
  return (
    <section className="run-detail-card run-detail-card--empty" aria-label="Run detail state">
      <strong>{title}</strong>
      <span>{detail}</span>
    </section>
  );
}

function hasPublicSourceLimitation(detail: RunDetail): boolean {
  const source = `${detail.provider.source ?? ""} ${detail.provider.provider_adapter ?? ""}`.toLowerCase();
  return detail.provider.package_used === false && (source.includes("public") || source.includes("fallback"));
}

function SummaryCards({ detail }: { detail: RunDetail }): JSX.Element {
  const missingMoneyFlow =
    !detail.financial_interpretation.where_money_comes_from &&
    !detail.financial_interpretation.where_money_goes &&
    !detail.financial_interpretation.cash_flow_explanation;
  const providerWarning = detail.provider.mock === true || hasPublicSourceLimitation(detail);

  return (
    <div className="summary-layout">
      <DetailSection badge={detail.ai_usage.local_mock_used ? "LOCAL_MOCK" : detail.ai_usage.external_ai_used ? "EXTERNAL_AI" : "UNKNOWN"} title="AI Source">
        {detail.ai_usage.local_mock_used ? (
          <p className="warning-copy">Local mock was used. Treat this run as non-external analysis.</p>
        ) : null}
        <dl className="detail-kv-grid">
          <KeyValueRow label="Source" value={detail.ai_usage.source ?? "unknown"} />
          <KeyValueRow label="External AI" value={booleanLabel(detail.ai_usage.external_ai_used)} />
          <KeyValueRow label="Local mock" value={booleanLabel(detail.ai_usage.local_mock_used)} />
          <KeyValueRow label="New calls" value={detail.ai_usage.new_external_ai_calls ?? "unknown"} />
          <KeyValueRow label="Cache hits" value={detail.ai_usage.cache_hits ?? "unknown"} />
          <KeyValueRow label="Model" value={detail.ai_usage.model ?? "unknown"} />
        </dl>
      </DetailSection>

      <DetailSection badge={providerWarning ? "WARNING" : "PASS"} title="Provider">
        {detail.provider.mock ? <p className="warning-copy">Provider metadata says mock data was used.</p> : null}
        {hasPublicSourceLimitation(detail) ? (
          <p className="warning-copy">Public endpoint or fallback source. Important values need manual verification.</p>
        ) : null}
        <dl className="detail-kv-grid">
          <KeyValueRow label="Provider" value={detail.provider.provider ?? "unknown"} />
          <KeyValueRow label="Source" value={detail.provider.source ?? "unknown"} />
          <KeyValueRow label="Adapter" value={detail.provider.provider_adapter ?? "unknown"} />
          <KeyValueRow label="Package" value={booleanLabel(detail.provider.package_used)} />
          <KeyValueRow label="Mock" value={booleanLabel(detail.provider.mock)} />
          <KeyValueRow label="Market/Currency" value={`${detail.provider.market ?? "unknown"} / ${detail.provider.currency ?? "unknown"}`} />
        </dl>
      </DetailSection>

      <DetailSection badge={statusToBadge(detail.company.confidence)} title="Company Identity">
        <dl className="detail-kv-grid">
          <KeyValueRow label="Name" value={detail.company.name ?? "unknown"} />
          <KeyValueRow label="Frame" value={detail.company.frame ?? "unknown"} />
          <KeyValueRow label="Confidence" value={detail.company.confidence ?? "unknown"} />
        </dl>
        <p className="detail-copy">{detail.company.identity ?? "No company identity loaded."}</p>
        <div className="subsection">
          <span className="subsection-title">Not this</span>
          <BulletList emptyLabel="No not-this boundaries loaded." items={detail.company.not_this} />
        </div>
      </DetailSection>

      <DetailSection badge={missingMoneyFlow ? "DATA_GAP" : "UNKNOWN"} title="Money Flow">
        {missingMoneyFlow ? <p className="warning-copy">Money flow fields are missing or incomplete.</p> : null}
        <dl className="detail-kv-grid detail-kv-grid--wide">
          <KeyValueRow label="Comes from" value={detail.financial_interpretation.where_money_comes_from ?? "unknown"} />
          <KeyValueRow label="Goes to" value={detail.financial_interpretation.where_money_goes ?? "unknown"} />
          <KeyValueRow label="Debt / financing" value={detail.financial_interpretation.debt_and_financing ?? "unknown"} />
          <KeyValueRow label="Valuation fit" value={detail.financial_interpretation.valuation_method_fit ?? "unknown"} />
        </dl>
        <p className="detail-copy">{detail.financial_interpretation.cash_flow_explanation ?? "No cash-flow explanation loaded."}</p>
      </DetailSection>

      <DetailSection badge="UNKNOWN" title="Blueprint">
        <p className="detail-copy">{detail.blueprint.core_thesis ?? "No core thesis loaded."}</p>
        <div className="split-lists">
          <div>
            <span className="subsection-title">Must analyze</span>
            <BulletList emptyLabel="No must-analyze items loaded." items={detail.blueprint.must_analyze} />
          </div>
          <div>
            <span className="subsection-title">Must not analyze as core</span>
            <BulletList emptyLabel="No must-not-analyze items loaded." items={detail.blueprint.must_not_analyze_as_core} />
          </div>
          <div>
            <span className="subsection-title">Next checks</span>
            <BulletList emptyLabel="No next checks loaded." items={detail.blueprint.next_checks} />
          </div>
        </div>
      </DetailSection>
    </div>
  );
}

function validatorAlerts(detail: RunDetail): string[] {
  const alerts = new Set<string>();
  const combined = [
    ...detail.warnings,
    detail.status.overall_status,
    detail.status.provider_status,
    detail.status.visual_lint_status,
    detail.self_review.framework_fit_check,
    detail.self_review.money_flow_check,
    detail.self_review.numeric_consistency_check
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();

  const alertRules: Array<[string, string[]]> = [
    ["Wrong framework risk", ["wrong framework", "framework conflict"]],
    ["Hallucinated revenue engine risk", ["hallucinated revenue", "unsupported revenue"]],
    ["Unsupported claim risk", ["unsupported claim", "unsupported numeric"]],
    ["Provider data gap", ["provider data gap", "missing field", "data gap"]],
    ["Missing AI provenance", ["missing ai provenance", "ai provenance missing"]],
    ["Local/mock AI warning", ["local mock", "local/mock"]]
  ];

  for (const [label, needles] of alertRules) {
    if (needles.some((needle) => combined.includes(needle))) {
      alerts.add(label);
    }
  }
  if (detail.status.human_review_required || detail.self_review.human_review_required) {
    alerts.add("Human review required");
  }
  if (detail.ai_usage.local_mock_used) {
    alerts.add("Local/mock AI warning");
  }
  if (detail.provider.missing_fields.length > 0 || detail.blueprint.data_gaps.length > 0) {
    alerts.add("Provider data gap");
  }

  return Array.from(alerts);
}

function AuditTrailPanel({ detail }: { detail: RunDetail }): JSX.Element {
  const [message, setMessage] = useState<string>("Audit trail reconstructed from completed-run metadata.");
  const [isBusy, setIsBusy] = useState<string | null>(null);
  const alerts = validatorAlerts(detail);

  async function handleOpen(stage: string, artifactPath: string | null): Promise<void> {
    if (!artifactPath) {
      return;
    }
    setIsBusy(stage);
    setMessage(`Opening ${stage} artifact...`);
    try {
      const result = await openArtifact(artifactPath);
      setMessage(result.ok ? `${stage} artifact opened.` : `${stage} artifact action returned a warning.`);
    } catch (error: unknown) {
      const text = error instanceof Error ? error.message : String(error);
      setMessage(text.includes("__TAURI__") ? "Desktop runtime required for audit artifact actions." : text);
    } finally {
      setIsBusy(null);
    }
  }

  return (
    <DetailSection badge={alerts.length > 0 ? "WARNING" : "UNKNOWN"} title="Audit Trail">
      <div className="audit-timeline" aria-label="Static audit trail">
        {detail.audit_trail.map((stage) => (
          <div className="audit-stage" key={stage.stage}>
            <div className="audit-stage__marker" aria-hidden="true" />
            <div className="audit-stage__body">
              <div className="audit-stage__topline">
                <div>
                  <strong>{stage.label}</strong>
                  <span>{stage.source ?? "source unknown"}</span>
                </div>
                <AuditStatusBadge status={stage.status} />
              </div>
              <p>{stage.message ?? "No stage message available."}</p>
              <button
                className="audit-open-button"
                disabled={!stage.artifact_path || isBusy !== null}
                onClick={() => void handleOpen(stage.label, stage.artifact_path)}
                type="button"
              >
                {stage.artifact_path ? "Open" : "No artifact"}
              </button>
            </div>
          </div>
        ))}
      </div>
      {alerts.length > 0 ? (
        <div className="validator-alerts">
          <span className="subsection-title">Validator alerts</span>
          <BulletList emptyLabel="No validator alerts." items={alerts} />
        </div>
      ) : null}
      <p className="artifact-message">{message}</p>
    </DetailSection>
  );
}

function chartLimitation(chart: RunDetail["charts"][number]): string {
  return (
    chart.what_not_to_overread ??
    (chart.image_exists
      ? "This chart is a visual aid only and does not create a buy/sell signal."
      : "Chart image missing, manifest available.")
  );
}

function ChartGrid({ detail }: { detail: RunDetail }): JSX.Element {
  const { t } = useTranslation();
  const [message, setMessage] = useState<string>("Chart grid uses existing artifacts only.");
  const [isBusy, setIsBusy] = useState<string | null>(null);

  async function handleOpen(title: string, imagePath: string | null): Promise<void> {
    if (!imagePath) {
      return;
    }
    setIsBusy(title);
    setMessage(`Opening chart artifact: ${title}...`);
    try {
      const result = await openArtifact(imagePath);
      setMessage(result.ok ? `${title} chart opened.` : `${title} chart action returned a warning.`);
    } catch (error: unknown) {
      const text = error instanceof Error ? error.message : String(error);
      setMessage(text.includes("__TAURI__") ? "Desktop runtime required for chart opening." : text);
    } finally {
      setIsBusy(null);
    }
  }

  if (detail.charts.length === 0) {
    return (
      <DetailSection badge="DATA_GAP" title={t("charts")}>
        <div className="chart-empty-state">
          <strong>No chart manifest found for this run.</strong>
          <span>The studio will only display existing chart artifacts; it does not generate charts.</span>
        </div>
      </DetailSection>
    );
  }

  return (
    <DetailSection badge="UNKNOWN" title={`${t("charts")} (${detail.charts.length})`}>
      <div className="chart-gallery">
        {detail.charts.map((chart) => (
          <article className="chart-card" key={`${chart.title}-${chart.image_path ?? "missing"}`}>
            <div className="chart-card__header">
              <strong>{chart.title}</strong>
              <StatusBadge variant={chart.image_exists ? statusToBadge(chart.status) : "WARNING"} />
            </div>
            {chart.image_exists && chart.image_path ? (
              <img alt={`${chart.title} chart preview`} className="chart-preview" src={artifactImageSrc(chart.image_path)} />
            ) : (
              <div className="chart-missing">Chart image missing, manifest available.</div>
            )}
            <dl className="chart-facts">
              <KeyValueRow label={t("source")} value={chart.source ?? "unknown"} />
              <KeyValueRow label="Look at" value={chart.what_to_look_at ?? "not specified"} />
              <KeyValueRow label="Means" value={chart.what_it_means ?? "not specified"} />
              <KeyValueRow label="Do not overread" value={chartLimitation(chart)} />
              <KeyValueRow label={t("nextCheck")} value={chart.next_check ?? "not specified"} />
            </dl>
            {chart.why_selected ? <p className="detail-copy">{chart.why_selected}</p> : null}
            <button
              className="artifact-button chart-open-button"
              disabled={!chart.image_exists || !chart.image_path || isBusy !== null}
              onClick={() => void handleOpen(chart.title, chart.image_path)}
              type="button"
            >
              <span>Open Chart</span>
              <small>{chart.image_exists ? "open" : "missing"}</small>
            </button>
          </article>
        ))}
      </div>
      <p className="artifact-message">{message}</p>
    </DetailSection>
  );
}

function DataGapsPanel({ detail }: { detail: RunDetail }): JSX.Element {
  const humanReview = detail.status.human_review_required || detail.self_review.human_review_required;
  const hasWarnings = detail.warnings.length > 0 || detail.provider.missing_fields.length > 0 || detail.blueprint.data_gaps.length > 0;

  return (
    <DetailSection badge={humanReview ? "FAIL" : hasWarnings ? "WARNING" : "PASS"} title="Data Gaps">
      {humanReview ? <p className="warning-copy">Human review is required or flagged by self-review/status metadata.</p> : null}
      <div className="split-lists split-lists--three">
        <div>
          <span className="subsection-title">Data gaps</span>
          <BulletList emptyLabel="No data gaps loaded." items={detail.blueprint.data_gaps} />
        </div>
        <div>
          <span className="subsection-title">Missing fields</span>
          <BulletList emptyLabel="No missing provider fields reported." items={detail.provider.missing_fields} />
        </div>
        <div>
          <span className="subsection-title">Loader warnings</span>
          <BulletList emptyLabel="No loader warnings." items={detail.warnings} />
        </div>
      </div>
      <div className="self-review-grid">
        <KeyValueRow label="Framework fit" value={detail.self_review.framework_fit_check ?? "unknown"} />
        <KeyValueRow label="Numeric consistency" value={detail.self_review.numeric_consistency_check ?? "unknown"} />
        <KeyValueRow label="Money flow check" value={detail.self_review.money_flow_check ?? "unknown"} />
        <KeyValueRow label="Final confidence" value={detail.self_review.final_confidence ?? "unknown"} />
      </div>
    </DetailSection>
  );
}

type ArtifactButtonConfig = {
  label: string;
  path: string | null;
  action: "open" | "reveal";
};

function ArtifactLinksPanel({ detail }: { detail: RunDetail }): JSX.Element {
  const [message, setMessage] = useState<string>("No artifact action yet.");
  const [isBusy, setIsBusy] = useState<string | null>(null);

  const artifactButtons: ArtifactButtonConfig[] = [
    { label: "Report", path: detail.artifacts.markdown_report_path, action: "open" },
    { label: "Dashboard", path: detail.artifacts.dashboard_path, action: "open" },
    { label: "PDF", path: detail.artifacts.pdf_report_path, action: "open" },
    { label: "AI Usage", path: detail.artifacts.ai_usage_path, action: "open" },
    { label: "Blueprint", path: detail.artifacts.blueprint_path, action: "open" },
    { label: "Validator", path: detail.artifacts.validator_report_path, action: "open" },
    { label: "Provider Payload", path: detail.artifacts.provider_payload_path, action: "open" },
    { label: "Run Folder", path: detail.run_folder, action: "reveal" }
  ];

  async function handleArtifactAction(config: ArtifactButtonConfig): Promise<void> {
    if (!config.path) {
      return;
    }

    setIsBusy(config.label);
    setMessage(`${config.action === "open" ? "Opening" : "Revealing"} ${config.label}...`);
    try {
      const result =
        config.action === "open" ? await openArtifact(config.path) : await revealInFolder(config.path);
      setMessage(result.ok ? `${config.label} ${config.action === "open" ? "opened" : "revealed"}.` : `${config.label} action returned a warning.`);
    } catch (error: unknown) {
      const text = error instanceof Error ? error.message : String(error);
      setMessage(text.includes("__TAURI__") ? "Desktop runtime required for artifact actions." : text);
    } finally {
      setIsBusy(null);
    }
  }

  return (
    <DetailSection badge="UNKNOWN" title="Artifact Dock">
      <div className="artifact-button-grid">
        {artifactButtons.map((config) => (
          <button
            className="artifact-button artifact-button--large"
            disabled={!config.path || isBusy !== null}
            key={config.label}
            onClick={() => void handleArtifactAction(config)}
            title={config.path ? `${config.label} artifact is available` : `${config.label} artifact is missing`}
            type="button"
          >
            <span>{config.label}</span>
            <small>{config.path ? config.action : "missing"}</small>
          </button>
        ))}
      </div>
      <p className="artifact-message">{message}</p>
    </DetailSection>
  );
}

export function RunDetailPanel({ activeTab, detail, error, status }: RunDetailPanelProps): JSX.Element {
  if (status === "idle") {
    return <EmptyRunDetailState title="No run selected" detail="Select a run from the left rail to load structured run metadata." />;
  }

  if (status === "loading") {
    return <EmptyRunDetailState title="Loading run detail" detail="Reading structured metadata through Tauri IPC." />;
  }

  if (status === "browser-preview" || status === "error" || !detail) {
    return (
      <EmptyRunDetailState
        title={status === "browser-preview" ? "Detail loading needs Tauri" : "Run detail failed"}
        detail={error ?? "The load_run_detail command returned an error."}
      />
    );
  }

  if (activeTab === "charts") {
    return <ChartGrid detail={detail} />;
  }
  if (activeTab === "audit") {
    return <AuditTrailPanel detail={detail} />;
  }
  if (activeTab === "gaps") {
    return <DataGapsPanel detail={detail} />;
  }
  if (activeTab === "artifacts") {
    return <ArtifactLinksPanel detail={detail} />;
  }
  return <SummaryCards detail={detail} />;
}
