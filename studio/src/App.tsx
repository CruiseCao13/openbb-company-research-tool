import { type ReactNode, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppInfoCard } from "./components/AppInfoCard";
import { getAppInfo, listRuns, loadRunDetail } from "./lib/tauri";
import type { AppInfo, AppInfoStatus, RunDetail, RunDetailStatus, RunsStatus, RunSummary } from "./types/app";

type StudioPing = {
  status: "ok";
  message: string;
};

type BadgeVariant = "PASS" | "WARNING" | "FAIL" | "DATA_GAP" | "EXTERNAL_AI" | "LOCAL_MOCK" | "UNKNOWN";

type ResearchCardConfig = {
  title: string;
  badge: BadgeVariant;
  body: string;
};

const placeholderCards: ResearchCardConfig[] = [
  {
    title: "Report Status",
    badge: "UNKNOWN",
    body: "Placeholder only. No report status has been loaded from a v5 run folder."
  },
  {
    title: "AI Source",
    badge: "EXTERNAL_AI",
    body: "Placeholder only. Future DTOs will show external, local, cache, and skipped provenance."
  },
  {
    title: "Company Identity",
    badge: "UNKNOWN",
    body: "Placeholder only. Company profile and research frame are intentionally not loaded in Phase 2."
  },
  {
    title: "Money Flow",
    badge: "DATA_GAP",
    body: "Placeholder only. Sources, uses, cash-flow gaps, and financing pressure will appear after run loading exists."
  },
  {
    title: "Data Gaps",
    badge: "WARNING",
    body: "Placeholder only. Missing provider fields and unsupported claims will be rendered from typed DTOs later."
  }
];

function StatusBadge({ variant }: { variant: BadgeVariant }): JSX.Element {
  return <span className={`status-badge status-badge--${variant.toLowerCase()}`}>{variant}</span>;
}

function statusToBadge(status: string | null): BadgeVariant {
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

function aiSourceBadge(run: RunSummary): BadgeVariant {
  if (run.external_ai_used) {
    return "EXTERNAL_AI";
  }
  if (run.local_mock_used) {
    return "LOCAL_MOCK";
  }
  return "UNKNOWN";
}

function runKey(run: RunSummary): string {
  return `${run.ticker}::${run.run_id}`;
}

function EmptyState({ title, detail }: { title: string; detail: string }): JSX.Element {
  return (
    <div className="empty-state">
      <strong>{title}</strong>
      <span>{detail}</span>
    </div>
  );
}

function RunList({
  error,
  runs,
  selectedRunKey,
  status,
  onSelectRun
}: {
  error: string | null;
  runs: RunSummary[];
  selectedRunKey: string | null;
  status: RunsStatus;
  onSelectRun: (run: RunSummary) => void;
}): JSX.Element {
  if (status === "loading") {
    return <EmptyState title="Loading runs..." detail="Scanning v5 reports folders through Tauri IPC" />;
  }

  if (status === "browser-preview") {
    return (
      <EmptyState
        title="Run discovery needs Tauri"
        detail="Browser preview cannot access the Rust IPC command. Launch the desktop shell to list runs."
      />
    );
  }

  if (status === "failed") {
    return <EmptyState title="Run discovery failed" detail={error ?? "The list_runs command returned an error."} />;
  }

  if (runs.length === 0) {
    return <EmptyState title="No runs found" detail="No reports/TICKER/runs/RUN_ID folders were discovered." />;
  }

  return (
    <div className="run-list" role="list">
      {runs.map((run) => {
        const key = runKey(run);
        return (
          <button
            className={`run-list-item${selectedRunKey === key ? " run-list-item--selected" : ""}`}
            key={key}
            onClick={() => onSelectRun(run)}
            type="button"
          >
            <span className="run-list-item__topline">
              <strong>{run.ticker}</strong>
              <StatusBadge variant={statusToBadge(run.status)} />
            </span>
            <span className="run-list-item__run-id">{run.run_id}</span>
            <span className="run-list-item__meta">
              {run.market ?? "market unknown"} / {run.provider ?? "provider unknown"}
            </span>
            <span className="run-list-item__badges">
              <StatusBadge variant={aiSourceBadge(run)} />
              {run.human_review_required ? <StatusBadge variant="WARNING" /> : null}
            </span>
          </button>
        );
      })}
    </div>
  );
}

function Sidebar({
  error,
  runs,
  runsStatus,
  selectedRunKey,
  onSelectRun
}: {
  error: string | null;
  runs: RunSummary[];
  runsStatus: RunsStatus;
  selectedRunKey: string | null;
  onSelectRun: (run: RunSummary) => void;
}): JSX.Element {
  return (
    <aside className="sidebar" aria-label="Run navigation">
      <div className="brand-block">
        <p className="eyebrow">v6 studio shell</p>
        <h1>v6 Tauri Research Studio</h1>
        <p>Desktop research workspace for v5 run folders</p>
      </div>

      <section className="panel">
        <div className="panel-header">
          <span>Runs</span>
          <StatusBadge variant={runsStatus === "failed" ? "WARNING" : "UNKNOWN"} />
        </div>
        <RunList
          error={error}
          runs={runs}
          selectedRunKey={selectedRunKey}
          status={runsStatus}
          onSelectRun={onSelectRun}
        />
      </section>
    </aside>
  );
}

function TopStatusStrip({ ipcMessage }: { ipcMessage: string }): JSX.Element {
  return (
    <header className="top-status-strip" aria-label="Studio status">
      <div className="status-cluster">
        <span className="status-dot" aria-hidden="true" />
        <span>Studio shell ready</span>
      </div>
      <span>No external API used</span>
      <span>No run loaded</span>
      <span className="ipc-readout">{ipcMessage}</span>
    </header>
  );
}

function ResearchCard({ card, wide }: { card: ResearchCardConfig; wide: boolean }): JSX.Element {
  return (
    <article className={`detail-card${wide ? " detail-card--wide" : ""}`}>
      <div className="card-header">
        <span className="card-label">{card.title}</span>
        <StatusBadge variant={card.badge} />
      </div>
      <p>{card.body}</p>
    </article>
  );
}

function BottomProvenanceBar(): JSX.Element {
  return (
    <section className="bottom-bar" aria-label="Provenance and data gaps">
      <div className="bottom-bar__header">
        <h3>Provenance &amp; Data Gaps</h3>
        <StatusBadge variant="DATA_GAP" />
      </div>
      <div className="bottom-bar__grid">
        <div className="provenance-cell">
          <span>AI provenance</span>
          <p>AI provenance will appear here</p>
        </div>
        <div className="provenance-cell">
          <span>Data gaps</span>
          <p>Data gaps will appear here</p>
        </div>
      </div>
    </section>
  );
}

function ValueList({ emptyLabel, items }: { emptyLabel: string; items: string[] }): JSX.Element {
  if (items.length === 0) {
    return <p>{emptyLabel}</p>;
  }

  return (
    <ul className="compact-list">
      {items.slice(0, 5).map((item) => (
        <li key={item}>{item}</li>
      ))}
    </ul>
  );
}

function DetailField({ label, value }: { label: string; value: ReactNode }): JSX.Element {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function formatBoolean(value: boolean | null): string {
  if (value === null) {
    return "unknown";
  }
  return value ? "yes" : "no";
}

function RunDetailCards({
  detail,
  error,
  status
}: {
  detail: RunDetail | null;
  error: string | null;
  status: RunDetailStatus;
}): JSX.Element {
  if (status === "idle") {
    return (
      <section className="selected-run-summary" aria-label="Selected run summary">
        <EmptyState title="No run selected" detail="Select a run from the sidebar to load structured run metadata." />
      </section>
    );
  }

  if (status === "loading") {
    return (
      <section className="selected-run-summary" aria-label="Selected run summary">
        <EmptyState title="Loading run detail..." detail="Reading structured metadata through Tauri IPC." />
      </section>
    );
  }

  if (status === "browser-preview" || status === "error" || !detail) {
    return (
      <section className="selected-run-summary" aria-label="Selected run summary">
        <EmptyState
          title={status === "browser-preview" ? "Detail loading needs Tauri" : "Run detail failed"}
          detail={error ?? "The load_run_detail command returned an error."}
        />
      </section>
    );
  }

  return (
    <>
      <section className="selected-run-summary" aria-label="Run header card">
      <div className="card-header">
        <span className="card-label">Header</span>
        <StatusBadge variant={statusToBadge(detail.status.overall_status)} />
      </div>
      <dl className="selected-run-grid">
        <DetailField label="Ticker" value={detail.ticker} />
        <DetailField label="Run ID" value={detail.run_id} />
        <DetailField label="Overall status" value={detail.status.overall_status ?? "unknown"} />
        <DetailField
          label="Human review"
          value={detail.status.human_review_required === null ? "unknown" : detail.status.human_review_required ? "required" : "not flagged"}
        />
        <DetailField label="Charts" value={detail.charts.length} />
        <div>
          <dt>Artifacts</dt>
          <dd>
            report {detail.artifacts.markdown_report_path ? "yes" : "no"} / dashboard{" "}
            {detail.artifacts.dashboard_path ? "yes" : "no"} / pdf {detail.artifacts.pdf_report_path ? "yes" : "no"}
          </dd>
        </div>
      </dl>
    </section>

      <section className="selected-run-summary" aria-label="AI source card">
        <div className="card-header">
          <span className="card-label">AI Source</span>
          <StatusBadge variant={detail.ai_usage.external_ai_used ? "EXTERNAL_AI" : detail.ai_usage.local_mock_used ? "LOCAL_MOCK" : "UNKNOWN"} />
        </div>
        <dl className="selected-run-grid">
          <DetailField label="Source" value={detail.ai_usage.source ?? "unknown"} />
          <DetailField label="External AI" value={formatBoolean(detail.ai_usage.external_ai_used)} />
          <DetailField label="Local mock" value={formatBoolean(detail.ai_usage.local_mock_used)} />
          <DetailField label="New external calls" value={detail.ai_usage.new_external_ai_calls ?? "unknown"} />
          <DetailField label="Cache hits" value={detail.ai_usage.cache_hits ?? "unknown"} />
          <DetailField label="Model" value={detail.ai_usage.model ?? "unknown"} />
        </dl>
      </section>

      <section className="selected-run-summary" aria-label="Provider card">
        <div className="card-header">
          <span className="card-label">Provider</span>
          <StatusBadge variant={detail.provider.mock ? "WARNING" : "UNKNOWN"} />
        </div>
        <dl className="selected-run-grid">
          <DetailField label="Provider" value={detail.provider.provider ?? "unknown"} />
          <DetailField label="Source" value={detail.provider.source ?? "unknown"} />
          <DetailField label="Adapter" value={detail.provider.provider_adapter ?? "unknown"} />
          <DetailField label="Package used" value={formatBoolean(detail.provider.package_used)} />
          <DetailField label="Mock" value={formatBoolean(detail.provider.mock)} />
          <DetailField label="Market/Currency" value={`${detail.provider.market ?? "unknown"} / ${detail.provider.currency ?? "unknown"}`} />
        </dl>
        <ValueList emptyLabel="No provider limitations reported." items={detail.provider.limitations} />
      </section>

      <section className="selected-run-summary" aria-label="Company identity card">
        <div className="card-header">
          <span className="card-label">Company Identity</span>
          <StatusBadge variant="UNKNOWN" />
        </div>
        <dl className="selected-run-grid">
          <DetailField label="Name" value={detail.company.name ?? "unknown"} />
          <DetailField label="Frame" value={detail.company.frame ?? "unknown"} />
          <DetailField label="Confidence" value={detail.company.confidence ?? "unknown"} />
        </dl>
        <p>{detail.company.identity ?? "No company identity loaded."}</p>
        <ValueList emptyLabel="No not-this boundaries loaded." items={detail.company.not_this} />
      </section>

      <section className="selected-run-summary" aria-label="Money flow card">
        <div className="card-header">
          <span className="card-label">Money Flow</span>
          <StatusBadge variant="DATA_GAP" />
        </div>
        <dl className="selected-run-grid">
          <DetailField label="Comes from" value={detail.financial_interpretation.where_money_comes_from ?? "unknown"} />
          <DetailField label="Goes to" value={detail.financial_interpretation.where_money_goes ?? "unknown"} />
          <DetailField label="Debt / financing" value={detail.financial_interpretation.debt_and_financing ?? "unknown"} />
        </dl>
        <p>{detail.financial_interpretation.cash_flow_explanation ?? "No cash-flow explanation loaded."}</p>
      </section>

      <section className="selected-run-summary" aria-label="Blueprint card">
        <div className="card-header">
          <span className="card-label">Blueprint</span>
          <StatusBadge variant="UNKNOWN" />
        </div>
        <p>{detail.blueprint.core_thesis ?? "No core thesis loaded."}</p>
        <div className="split-lists">
          <ValueList emptyLabel="No must-analyze items loaded." items={detail.blueprint.must_analyze} />
          <ValueList emptyLabel="No must-not-analyze items loaded." items={detail.blueprint.must_not_analyze_as_core} />
          <ValueList emptyLabel="No next checks loaded." items={detail.blueprint.next_checks} />
        </div>
      </section>

      <section className="selected-run-summary" aria-label="Data gaps and warnings card">
        <div className="card-header">
          <span className="card-label">Data Gaps / Warnings</span>
          <StatusBadge variant={detail.warnings.length > 0 ? "WARNING" : "PASS"} />
        </div>
        <div className="split-lists">
          <ValueList emptyLabel="No data gaps loaded." items={detail.blueprint.data_gaps} />
          <ValueList emptyLabel="No missing provider fields reported." items={detail.provider.missing_fields} />
          <ValueList emptyLabel="No loader warnings." items={detail.warnings} />
        </div>
      </section>
    </>
  );
}

function AppShell({ children }: { children: ReactNode }): JSX.Element {
  return <main className="studio-shell">{children}</main>;
}

export function App(): JSX.Element {
  const [ipcMessage, setIpcMessage] = useState<string>("IPC not checked");
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [appInfoStatus, setAppInfoStatus] = useState<AppInfoStatus>("loading");
  const [appInfoError, setAppInfoError] = useState<string | null>(null);
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [runsStatus, setRunsStatus] = useState<RunsStatus>("loading");
  const [runsError, setRunsError] = useState<string | null>(null);
  const [selectedRunKey, setSelectedRunKey] = useState<string | null>(null);
  const [activeRunDetail, setActiveRunDetail] = useState<RunDetail | null>(null);
  const [runDetailStatus, setRunDetailStatus] = useState<RunDetailStatus>("idle");
  const [runDetailError, setRunDetailError] = useState<string | null>(null);

  const selectedRun = runs.find((run) => runKey(run) === selectedRunKey) ?? null;

  useEffect(() => {
    let mounted = true;

    invoke<StudioPing>("ping_studio")
      .then((response) => {
        if (mounted) {
          setIpcMessage(`${response.status}: ${response.message}`);
        }
      })
      .catch(() => {
        if (mounted) {
          setIpcMessage("IPC unavailable in browser preview");
        }
      });

    getAppInfo()
      .then((info) => {
        if (mounted) {
          setAppInfo(info);
          setAppInfoStatus("connected");
          setAppInfoError(null);
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          const message = error instanceof Error ? error.message : String(error);
          setAppInfo(null);
          setAppInfoStatus(message.includes("__TAURI__") ? "browser-preview" : "failed");
          setAppInfoError(
            message.includes("__TAURI__")
              ? "Tauri IPC is unavailable in browser preview. Run the desktop app to verify the command."
              : message
          );
        }
      });

    listRuns()
      .then((loadedRuns) => {
        if (mounted) {
          setRuns(loadedRuns);
          setRunsStatus(loadedRuns.length > 0 ? "ready" : "empty");
          setRunsError(null);
          setSelectedRunKey((current) => current ?? (loadedRuns[0] ? runKey(loadedRuns[0]) : null));
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          const message = error instanceof Error ? error.message : String(error);
          setRuns([]);
          setRunsStatus(message.includes("__TAURI__") ? "browser-preview" : "failed");
          setRunsError(
            message.includes("__TAURI__")
              ? "Tauri IPC is unavailable in browser preview. Real run discovery requires the desktop runtime."
              : message
          );
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    if (!selectedRun) {
      setActiveRunDetail(null);
      setRunDetailStatus("idle");
      setRunDetailError(null);
      return;
    }

    let mounted = true;
    setRunDetailStatus("loading");
    setRunDetailError(null);

    loadRunDetail(selectedRun.ticker, selectedRun.run_id)
      .then((detail) => {
        if (mounted) {
          setActiveRunDetail(detail);
          setRunDetailStatus("ready");
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          const message = error instanceof Error ? error.message : String(error);
          setActiveRunDetail(null);
          setRunDetailStatus(message.includes("__TAURI__") ? "browser-preview" : "error");
          setRunDetailError(
            message.includes("__TAURI__")
              ? "Tauri IPC is unavailable in browser preview. Real detail loading requires the desktop runtime."
              : message
          );
        }
      });

    return () => {
      mounted = false;
    };
  }, [selectedRun]);

  return (
    <AppShell>
      <Sidebar
        error={runsError}
        runs={runs}
        runsStatus={runsStatus}
        selectedRunKey={selectedRunKey}
        onSelectRun={(run) => setSelectedRunKey(runKey(run))}
      />

      <section className="workspace" aria-label="Research run detail">
        <TopStatusStrip ipcMessage={ipcMessage} />

        <header className="workspace-header">
          <div>
            <p className="eyebrow">Static placeholder layout</p>
            <h2>Research Run Detail</h2>
            <p>Select a run to inspect locked data, AI provenance, validator logs, and report artifacts.</p>
          </div>
        </header>

        <section className="card-grid" aria-label="Placeholder detail cards">
          <AppInfoCard appInfo={appInfo} error={appInfoError} status={appInfoStatus} />
          <RunDetailCards detail={activeRunDetail} error={runDetailError} status={runDetailStatus} />
          {placeholderCards.map((card, index) => (
            <ResearchCard card={card} key={card.title} wide={index >= 3} />
          ))}
        </section>

        <BottomProvenanceBar />
      </section>
    </AppShell>
  );
}
