import { type ReactNode, useState } from "react";
import { openArtifact, revealInFolder } from "../lib/tauri";
import type { RunDetail, RunDetailStatus } from "../types/app";

type DetailBadgeVariant = "PASS" | "WARNING" | "FAIL" | "DATA_GAP" | "EXTERNAL_AI" | "LOCAL_MOCK" | "UNKNOWN";

type RunDetailPanelProps = {
  detail: RunDetail | null;
  error: string | null;
  status: RunDetailStatus;
};

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

function StatusBadge({ variant }: { variant: DetailBadgeVariant }): JSX.Element {
  return <span className={`status-badge status-badge--${variant.toLowerCase()}`}>{variant}</span>;
}

function KeyValueRow({ label, value }: { label: string; value: ReactNode }): JSX.Element {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function BulletList({ emptyLabel, items }: { emptyLabel: string; items: string[] }): JSX.Element {
  if (items.length === 0) {
    return <p className="muted-copy">{emptyLabel}</p>;
  }

  return (
    <ul className="compact-list">
      {items.slice(0, 6).map((item) => (
        <li key={item}>{item}</li>
      ))}
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
  return "UNKNOWN";
}

function booleanLabel(value: boolean | null): string {
  if (value === null) {
    return "unknown";
  }
  return value ? "yes" : "no";
}

function humanReviewLabel(value: boolean | null): string {
  if (value === null) {
    return "unknown";
  }
  return value ? "required" : "not flagged";
}

function hasPublicSourceLimitation(detail: RunDetail): boolean {
  const source = `${detail.provider.source ?? ""} ${detail.provider.provider_adapter ?? ""}`.toLowerCase();
  return detail.provider.package_used === false && (source.includes("public") || source.includes("fallback"));
}

function warningBadge(enabled: boolean): DetailBadgeVariant {
  return enabled ? "WARNING" : "PASS";
}

function HeaderCard({ detail }: { detail: RunDetail }): JSX.Element {
  return (
    <DetailSection badge={statusToBadge(detail.status.overall_status)} title="Header">
      <div className="run-hero">
        <div>
          <p className="eyebrow">Selected run</p>
          <h3>{detail.ticker}</h3>
          <p className="mono-path">{detail.run_id}</p>
        </div>
        <div className="hero-badges">
          <StatusBadge variant={statusToBadge(detail.status.overall_status)} />
          {detail.status.human_review_required ? <StatusBadge variant="WARNING" /> : null}
        </div>
      </div>
      <dl className="detail-kv-grid">
        <KeyValueRow label="Overall status" value={detail.status.overall_status ?? "unknown"} />
        <KeyValueRow label="Human review" value={humanReviewLabel(detail.status.human_review_required)} />
        <KeyValueRow label="Provider status" value={detail.status.provider_status ?? "unknown"} />
        <KeyValueRow label="Visual lint" value={detail.status.visual_lint_status ?? "unknown"} />
        <KeyValueRow label="PDF export" value={detail.status.pdf_export_status ?? "unknown"} />
        <KeyValueRow label="Charts" value={detail.charts.length} />
      </dl>
      <p className="mono-path">{detail.run_folder}</p>
    </DetailSection>
  );
}

function AiSourceCard({ detail }: { detail: RunDetail }): JSX.Element {
  const badge = detail.ai_usage.local_mock_used
    ? "LOCAL_MOCK"
    : detail.ai_usage.external_ai_used
      ? "EXTERNAL_AI"
      : "UNKNOWN";

  return (
    <DetailSection badge={badge} title="AI Source">
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
      <BulletList emptyLabel="No prompt versions reported." items={detail.ai_usage.prompt_versions} />
    </DetailSection>
  );
}

function ProviderCard({ detail }: { detail: RunDetail }): JSX.Element {
  const hasWarning = detail.provider.mock === true || hasPublicSourceLimitation(detail);

  return (
    <DetailSection badge={warningBadge(hasWarning)} title="Provider">
      {detail.provider.mock ? <p className="warning-copy">Provider metadata says mock data was used.</p> : null}
      {hasPublicSourceLimitation(detail) ? (
        <p className="warning-copy">Public endpoint or fallback source. Important values need manual verification.</p>
      ) : null}
      <dl className="detail-kv-grid">
        <KeyValueRow label="Provider" value={detail.provider.provider ?? "unknown"} />
        <KeyValueRow label="Source" value={detail.provider.source ?? "unknown"} />
        <KeyValueRow label="Adapter" value={detail.provider.provider_adapter ?? "unknown"} />
        <KeyValueRow label="Package used" value={booleanLabel(detail.provider.package_used)} />
        <KeyValueRow label="Mock" value={booleanLabel(detail.provider.mock)} />
        <KeyValueRow label="Market/Currency" value={`${detail.provider.market ?? "unknown"} / ${detail.provider.currency ?? "unknown"}`} />
      </dl>
      <BulletList emptyLabel="No provider limitations reported." items={detail.provider.limitations} />
    </DetailSection>
  );
}

function CompanyIdentityCard({ detail }: { detail: RunDetail }): JSX.Element {
  return (
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
  );
}

function MoneyFlowCard({ detail }: { detail: RunDetail }): JSX.Element {
  const missingMoneyFlow =
    !detail.financial_interpretation.where_money_comes_from &&
    !detail.financial_interpretation.where_money_goes &&
    !detail.financial_interpretation.cash_flow_explanation;

  return (
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
  );
}

function BlueprintCard({ detail }: { detail: RunDetail }): JSX.Element {
  return (
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
  );
}

function DataGapsCard({ detail }: { detail: RunDetail }): JSX.Element {
  const humanReview = detail.status.human_review_required || detail.self_review.human_review_required;
  const hasWarnings = detail.warnings.length > 0 || detail.provider.missing_fields.length > 0 || detail.blueprint.data_gaps.length > 0;

  return (
    <DetailSection badge={humanReview ? "FAIL" : hasWarnings ? "WARNING" : "PASS"} title="Data Gaps / Warnings">
      {humanReview ? <p className="warning-copy">Human review is required or flagged by self-review/status metadata.</p> : null}
      <div className="split-lists">
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
    </DetailSection>
  );
}

type ArtifactButtonConfig = {
  label: string;
  path: string | null;
  action: "open" | "reveal";
};

function ArtifactLinksCard({ detail }: { detail: RunDetail }): JSX.Element {
  const [message, setMessage] = useState<string>("No artifact action yet.");
  const [isBusy, setIsBusy] = useState<string | null>(null);

  const artifactButtons: ArtifactButtonConfig[] = [
    { label: "Open Markdown Report", path: detail.artifacts.markdown_report_path, action: "open" },
    { label: "Open Dashboard", path: detail.artifacts.dashboard_path, action: "open" },
    { label: "Open PDF", path: detail.artifacts.pdf_report_path, action: "open" },
    { label: "Open AI Usage", path: detail.artifacts.ai_usage_path, action: "open" },
    { label: "Open Research Blueprint", path: detail.artifacts.blueprint_path, action: "open" },
    { label: "Open Validator Report", path: detail.artifacts.validator_report_path, action: "open" },
    { label: "Open Provider Payload", path: detail.artifacts.provider_payload_path, action: "open" },
    { label: "Reveal Run Folder", path: detail.run_folder, action: "reveal" }
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
      setMessage(`${result.message}: ${result.path}`);
    } catch (error: unknown) {
      const text = error instanceof Error ? error.message : String(error);
      const browserPreview = text.includes("__TAURI__")
        ? "Tauri IPC is unavailable in browser preview. Artifact opening requires the desktop runtime."
        : text;
      setMessage(`Artifact action failed: ${browserPreview}`);
    } finally {
      setIsBusy(null);
    }
  }

  return (
    <DetailSection badge="UNKNOWN" title="Artifacts">
      <div className="artifact-button-grid">
        {artifactButtons.map((config) => (
          <button
            className="artifact-button"
            disabled={!config.path || isBusy !== null}
            key={config.label}
            onClick={() => void handleArtifactAction(config)}
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

export function RunDetailPanel({ detail, error, status }: RunDetailPanelProps): JSX.Element {
  if (status === "idle") {
    return <EmptyRunDetailState title="No run selected" detail="Select a run from the sidebar to load structured run metadata." />;
  }

  if (status === "loading") {
    return <EmptyRunDetailState title="Loading run detail..." detail="Reading structured metadata through Tauri IPC." />;
  }

  if (status === "browser-preview" || status === "error" || !detail) {
    return (
      <EmptyRunDetailState
        title={status === "browser-preview" ? "Detail loading needs Tauri" : "Run detail failed"}
        detail={error ?? "The load_run_detail command returned an error."}
      />
    );
  }

  return (
    <>
      <HeaderCard detail={detail} />
      <AiSourceCard detail={detail} />
      <ProviderCard detail={detail} />
      <CompanyIdentityCard detail={detail} />
      <MoneyFlowCard detail={detail} />
      <BlueprintCard detail={detail} />
      <DataGapsCard detail={detail} />
      <ArtifactLinksCard detail={detail} />
    </>
  );
}
