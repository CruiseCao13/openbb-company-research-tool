import { type ReactNode, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppInfoCard } from "./components/AppInfoCard";
import { RunDetailPanel, type RunDetailTab } from "./components/RunDetailPanel";
import {
  getAppInfo,
  listRuns,
  listTrainingRuns,
  loadQualityMatrix,
  loadRunDetail,
  openArtifact,
  revealInFolder
} from "./lib/tauri";
import type {
  AppInfo,
  AppInfoStatus,
  QualityMatrix,
  QualityMatrixRow,
  RunDetail,
  RunDetailStatus,
  RunsStatus,
  RunSummary,
  TrainingRunSummary
} from "./types/app";

type StudioPing = {
  status: "ok";
  message: string;
};

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

type StudioMode = "runs" | "matrix";
type TrainingRunsStatus = "idle" | "loading" | "ready" | "empty" | "failed" | "browser-preview";
type MatrixStatus = "idle" | "loading" | "ready" | "empty" | "failed" | "browser-preview";

const detailTabs: Array<{ id: RunDetailTab; label: string }> = [
  { id: "summary", label: "Summary" },
  { id: "charts", label: "Charts" },
  { id: "audit", label: "Audit Trail" },
  { id: "gaps", label: "Data Gaps" },
  { id: "artifacts", label: "Artifacts" }
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
  if (normalized.includes("GAP")) {
    return "DATA_GAP";
  }
  return "UNKNOWN";
}

function aiSourceBadge(run: RunSummary | null): BadgeVariant {
  if (run?.external_ai_used) {
    return "EXTERNAL_AI";
  }
  if (run?.local_mock_used) {
    return "LOCAL_MOCK";
  }
  return "UNKNOWN";
}

function runKey(run: RunSummary): string {
  return `${run.ticker}::${run.run_id}`;
}

function shortRunId(runId: string): string {
  return runId.length > 28 ? `${runId.slice(0, 25)}...` : runId;
}

function booleanLabel(value: boolean | null): string {
  if (value === null) {
    return "unknown";
  }
  return value ? "yes" : "no";
}

function EmptyState({ detail, title }: { detail: string; title: string }): JSX.Element {
  return (
    <div className="empty-state">
      <strong>{title}</strong>
      <span>{detail}</span>
    </div>
  );
}

function countDataGaps(detail: RunDetail | null): number {
  if (!detail) {
    return 0;
  }
  return new Set([...detail.blueprint.data_gaps, ...detail.provider.missing_fields]).size;
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

function artifactCount(detail: RunDetail | null): { available: number; total: number } {
  if (!detail) {
    return { available: 0, total: 7 };
  }
  const artifactPaths = [
    detail.artifacts.markdown_report_path,
    detail.artifacts.dashboard_path,
    detail.artifacts.pdf_report_path,
    detail.artifacts.ai_usage_path,
    detail.artifacts.blueprint_path,
    detail.artifacts.validator_report_path,
    detail.artifacts.provider_payload_path
  ];
  return {
    available: artifactPaths.filter(Boolean).length,
    total: artifactPaths.length + 1
  };
}

function Sidebar({
  error,
  mode,
  onChangeMode,
  onSearch,
  onSelectRun,
  runs,
  runsStatus,
  search,
  selectedRunKey
}: {
  error: string | null;
  mode: StudioMode;
  onChangeMode: (mode: StudioMode) => void;
  onSearch: (search: string) => void;
  onSelectRun: (run: RunSummary) => void;
  runs: RunSummary[];
  runsStatus: RunsStatus;
  search: string;
  selectedRunKey: string | null;
}): JSX.Element {
  return (
    <aside className="nav-rail" aria-label="Studio navigation">
      <div className="brand-block">
        <p className="eyebrow">v6 desktop studio</p>
        <h1>Research Studio</h1>
        <span>Browse v5 run folders</span>
      </div>

      <div className="mode-switch" role="tablist" aria-label="Workspace mode">
        <button
          className={mode === "runs" ? "mode-switch__button mode-switch__button--active" : "mode-switch__button"}
          onClick={() => onChangeMode("runs")}
          type="button"
        >
          Runs
        </button>
        <button
          className={mode === "matrix" ? "mode-switch__button mode-switch__button--active" : "mode-switch__button"}
          onClick={() => onChangeMode("matrix")}
          type="button"
        >
          Matrix
        </button>
      </div>

      <label className="run-search">
        <span>Filter runs</span>
        <input
          onChange={(event) => onSearch(event.target.value)}
          placeholder="Ticker or run id"
          type="search"
          value={search}
        />
      </label>

      <section className="run-list-panel" aria-label="Runs">
        <div className="rail-section-header">
          <span>Runs</span>
          <small>{runs.length}</small>
        </div>
        <RunList
          error={error}
          runs={runs}
          selectedRunKey={selectedRunKey}
          status={runsStatus}
          onSelectRun={(run) => {
            onChangeMode("runs");
            onSelectRun(run);
          }}
        />
      </section>
    </aside>
  );
}

function RunList({
  error,
  onSelectRun,
  runs,
  selectedRunKey,
  status
}: {
  error: string | null;
  onSelectRun: (run: RunSummary) => void;
  runs: RunSummary[];
  selectedRunKey: string | null;
  status: RunsStatus;
}): JSX.Element {
  if (status === "loading") {
    return <EmptyState title="Loading runs" detail="Scanning through Tauri IPC." />;
  }
  if (status === "browser-preview") {
    return <EmptyState title="Desktop required" detail="Real run discovery needs Tauri runtime." />;
  }
  if (status === "failed") {
    return <EmptyState title="Run discovery failed" detail={error ?? "list_runs returned an error."} />;
  }
  if (runs.length === 0) {
    return <EmptyState title="No runs found" detail="No matching reports/TICKER/runs folders." />;
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
            <span className="run-list-item__run-id">{shortRunId(run.run_id)}</span>
            <span className="run-list-item__meta">
              {run.market ?? "market unknown"} / {run.provider ?? "provider unknown"}
            </span>
            <span className="run-list-item__badges">
              <StatusBadge variant={aiSourceBadge(run)} />
              {run.human_review_required ? <StatusBadge variant="HUMAN_REVIEW" /> : null}
            </span>
          </button>
        );
      })}
    </div>
  );
}

function PrimaryActionBar({ detail }: { detail: RunDetail | null }): JSX.Element {
  const [message, setMessage] = useState<string>("Choose an artifact action.");
  const [busyLabel, setBusyLabel] = useState<string | null>(null);

  const actions = [
    { label: "Open Report", path: detail?.artifacts.markdown_report_path ?? null, action: "open" as const },
    { label: "Open Dashboard", path: detail?.artifacts.dashboard_path ?? null, action: "open" as const },
    { label: "Open PDF", path: detail?.artifacts.pdf_report_path ?? null, action: "open" as const },
    { label: "Reveal Folder", path: detail?.run_folder ?? null, action: "reveal" as const },
    { label: "Open AI Usage", path: detail?.artifacts.ai_usage_path ?? null, action: "open" as const },
    { label: "Open Validator Audit", path: detail?.artifacts.validator_report_path ?? null, action: "open" as const },
    { label: "Open Provider Payload", path: detail?.artifacts.provider_payload_path ?? null, action: "open" as const }
  ];

  async function handleAction(label: string, path: string | null, action: "open" | "reveal"): Promise<void> {
    if (!path) {
      return;
    }
    setBusyLabel(label);
    setMessage(`${action === "open" ? "Opening" : "Revealing"} ${label}...`);
    try {
      const result = action === "open" ? await openArtifact(path) : await revealInFolder(path);
      setMessage(`${result.message}: ${result.path}`);
    } catch (error: unknown) {
      const text = error instanceof Error ? error.message : String(error);
      setMessage(text.includes("__TAURI__") ? "Desktop runtime required for artifact actions." : text);
    } finally {
      setBusyLabel(null);
    }
  }

  return (
    <section className="artifact-dock" aria-label="Primary artifact actions">
      <div className="artifact-dock__buttons">
        {actions.map((action) => (
          <button
            className="primary-action"
            disabled={!action.path || busyLabel !== null}
            key={action.label}
            onClick={() => void handleAction(action.label, action.path, action.action)}
            title={action.path ? action.path : `${action.label} unavailable for this run`}
            type="button"
          >
            <span>{action.label}</span>
            <small>{action.path ? action.action : "missing"}</small>
          </button>
        ))}
      </div>
      <p>{message}</p>
    </section>
  );
}

function RunWorkspaceHeader({
  detail,
  detailStatus,
  ipcMessage,
  selectedRun
}: {
  detail: RunDetail | null;
  detailStatus: RunDetailStatus;
  ipcMessage: string;
  selectedRun: RunSummary | null;
}): JSX.Element {
  const warnings = collectWarnings(detail);
  const chartCount = detail?.charts.length ?? 0;
  const artifacts = artifactCount(detail);

  return (
    <header className="workspace-hero">
      <div className="workspace-hero__identity">
        <p className="eyebrow">Report Workspace</p>
        <div className="workspace-hero__title-row">
          <h2>{selectedRun?.ticker ?? "No run selected"}</h2>
          {detail?.company.name ? <span>{detail.company.name}</span> : null}
        </div>
        <p className="mono-path">{selectedRun ? selectedRun.run_id : "Select a run from the rail"}</p>
      </div>
      <div className="workspace-hero__signals">
        <StatusBadge variant={detailStatus === "ready" ? statusToBadge(detail?.status.overall_status ?? null) : "UNKNOWN"} />
        {detail?.status.human_review_required || detail?.self_review.human_review_required ? (
          <StatusBadge variant="HUMAN_REVIEW" />
        ) : null}
        <StatusBadge variant={detail?.ai_usage.external_ai_used ? "EXTERNAL_AI" : detail?.ai_usage.local_mock_used ? "LOCAL_MOCK" : "UNKNOWN"} />
        {detail?.provider.mock ? <StatusBadge variant="PROVIDER_MOCK" /> : null}
        <span className="signal-pill">{detail?.provider.provider ?? selectedRun?.provider ?? "provider unknown"}</span>
        <span className="signal-pill">{detail?.provider.market ?? selectedRun?.market ?? "market unknown"}</span>
        <span className="signal-pill">charts {chartCount}</span>
        <span className="signal-pill">artifacts {artifacts.available}/{artifacts.total}</span>
        <span className="signal-pill signal-pill--warning">warnings {warnings.length}</span>
        <span className="ipc-readout">{ipcMessage}</span>
      </div>
    </header>
  );
}

function DetailTabs({
  activeTab,
  chartCount,
  dataGapCount,
  onChange,
  warningCount
}: {
  activeTab: RunDetailTab;
  chartCount: number;
  dataGapCount: number;
  onChange: (tab: RunDetailTab) => void;
  warningCount: number;
}): JSX.Element {
  function tabMeta(id: RunDetailTab): string | null {
    if (id === "charts") {
      return String(chartCount);
    }
    if (id === "gaps") {
      return String(dataGapCount + warningCount);
    }
    return null;
  }

  return (
    <nav className="detail-tabs" aria-label="Run detail sections">
      {detailTabs.map((tab) => (
        <button
          className={activeTab === tab.id ? "detail-tab detail-tab--active" : "detail-tab"}
          key={tab.id}
          onClick={() => onChange(tab.id)}
          type="button"
        >
          <span>{tab.label}</span>
          {tabMeta(tab.id) ? <small>{tabMeta(tab.id)}</small> : null}
        </button>
      ))}
    </nav>
  );
}

function DiagnosticsDrawer({
  detail,
  status
}: {
  detail: RunDetail | null;
  status: RunDetailStatus;
}): JSX.Element {
  const [expanded, setExpanded] = useState<boolean>(false);
  const warnings = collectWarnings(detail);
  const dataGapCount = countDataGaps(detail);
  const isReady = status === "ready" && detail !== null;

  return (
    <aside className={`diagnostics-drawer${expanded ? " diagnostics-drawer--expanded" : ""}`} aria-label="Diagnostics drawer">
      <button className="diagnostics-strip" onClick={() => setExpanded((current) => !current)} type="button">
        <span>Diagnostics</span>
        <StatusBadge variant={detail?.ai_usage.external_ai_used ? "EXTERNAL_AI" : detail?.ai_usage.local_mock_used ? "LOCAL_MOCK" : "UNKNOWN"} />
        <span>{detail?.provider.provider ?? "provider unknown"}</span>
        <span>warnings {warnings.length}</span>
        <span>data gaps {dataGapCount}</span>
        {detail?.status.human_review_required || detail?.self_review.human_review_required ? (
          <StatusBadge variant="HUMAN_REVIEW" />
        ) : null}
        <small>{expanded ? "Collapse" : "Expand"}</small>
      </button>
      {expanded ? (
        <div className="diagnostics-panel">
          {!isReady ? (
            <EmptyState title={status === "loading" ? "Loading diagnostics" : "No run selected"} detail="Select a run to inspect provenance and data gaps." />
          ) : (
            <>
              <section>
                <h3>AI Provenance</h3>
                <dl className="diagnostic-kv">
                  <div><dt>Source</dt><dd>{detail.ai_usage.source ?? "unknown"}</dd></div>
                  <div><dt>External AI</dt><dd>{booleanLabel(detail.ai_usage.external_ai_used)}</dd></div>
                  <div><dt>Local mock</dt><dd>{booleanLabel(detail.ai_usage.local_mock_used)}</dd></div>
                  <div><dt>New calls</dt><dd>{detail.ai_usage.new_external_ai_calls ?? "unknown"}</dd></div>
                  <div><dt>Cache hits</dt><dd>{detail.ai_usage.cache_hits ?? "unknown"}</dd></div>
                  <div><dt>Model</dt><dd>{detail.ai_usage.model ?? "unknown"}</dd></div>
                </dl>
              </section>
              <section>
                <h3>Provider</h3>
                <dl className="diagnostic-kv">
                  <div><dt>Provider</dt><dd>{detail.provider.provider ?? "unknown"}</dd></div>
                  <div><dt>Source</dt><dd>{detail.provider.source ?? "unknown"}</dd></div>
                  <div><dt>Adapter</dt><dd>{detail.provider.provider_adapter ?? "unknown"}</dd></div>
                  <div><dt>Package</dt><dd>{booleanLabel(detail.provider.package_used)}</dd></div>
                  <div><dt>Mock</dt><dd>{booleanLabel(detail.provider.mock)}</dd></div>
                  <div><dt>Market</dt><dd>{detail.provider.market ?? "unknown"} / {detail.provider.currency ?? "unknown"}</dd></div>
                </dl>
              </section>
              <section>
                <h3>Data Gaps & Warnings</h3>
                {warnings.length === 0 ? (
                  <p className="muted-copy">No data gaps, missing fields, or warnings reported.</p>
                ) : (
                  <ul className="compact-list">
                    {warnings.slice(0, 8).map((warning) => <li key={warning}>{warning}</li>)}
                    {warnings.length > 8 ? <li>{warnings.length - 8} more</li> : null}
                  </ul>
                )}
              </section>
            </>
          )}
        </div>
      ) : null}
    </aside>
  );
}

function scoreClass(row: QualityMatrixRow): string {
  const score = row.quality_score;
  if (score === null) {
    return "matrix-cell--unknown";
  }
  if (score >= 85) {
    return "matrix-cell--good";
  }
  if (score >= 70) {
    return "matrix-cell--ok";
  }
  if (score >= 60) {
    return "matrix-cell--weak";
  }
  return "matrix-cell--fail";
}

function averageQuality(rows: QualityMatrixRow[]): string {
  const scores = rows.map((row) => row.quality_score).filter((score): score is number => score !== null);
  if (scores.length === 0) {
    return "unknown";
  }
  return (scores.reduce((sum, score) => sum + score, 0) / scores.length).toFixed(1);
}

function countProviderFailures(rows: QualityMatrixRow[]): number {
  return rows.filter((row) => (row.provider_status ?? "").toLowerCase().includes("fail")).length;
}

function MatrixSummaryCard({ label, value }: { label: string; value: ReactNode }): JSX.Element {
  return (
    <div className="matrix-summary-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function RegressionMatrixHub({
  error,
  matrix,
  matrixError,
  matrixStatus,
  onSelectRow,
  onSelectRun,
  runs,
  selectedRow,
  selectedTrainingRunId,
  trainingStatus
}: {
  error: string | null;
  matrix: QualityMatrix | null;
  matrixError: string | null;
  matrixStatus: MatrixStatus;
  onSelectRow: (row: QualityMatrixRow) => void;
  onSelectRun: (runId: string) => void;
  runs: TrainingRunSummary[];
  selectedRow: QualityMatrixRow | null;
  selectedTrainingRunId: string | null;
  trainingStatus: TrainingRunsStatus;
}): JSX.Element {
  if (trainingStatus === "loading") {
    return <EmptyState title="Loading training runs" detail="Scanning reports/training_runs through Tauri IPC." />;
  }
  if (trainingStatus === "browser-preview") {
    return <EmptyState title="Desktop runtime required" detail="Regression Matrix needs Tauri IPC to read existing artifacts." />;
  }
  if (trainingStatus === "failed") {
    return <EmptyState title="Training run discovery failed" detail={error ?? "list_training_runs returned an error."} />;
  }
  if (runs.length === 0) {
    return <EmptyState title="No training runs found" detail="No reports/training_runs folders were discovered." />;
  }

  const rows = matrix?.rows ?? [];
  const hardFailures = rows.reduce((count, row) => count + row.hard_failures.length, 0);
  const providerFailures = countProviderFailures(rows);
  const warningsCount = (matrix?.warnings.length ?? 0) + rows.filter((row) => row.issue_types.length > 0).length;

  return (
    <section className="matrix-workspace" aria-label="Regression Matrix Hub">
      <header className="matrix-hero">
        <div>
          <p className="eyebrow">Quality control board</p>
          <h2>Regression Matrix Hub</h2>
          <p>Read existing training run quality matrices. No training or external API calls are performed.</p>
        </div>
        <label className="training-selector">
          <span>Training run</span>
          <select
            aria-label="Training run selector"
            onChange={(event) => onSelectRun(event.target.value)}
            value={selectedTrainingRunId ?? ""}
          >
            {runs.map((run) => (
              <option key={run.run_id} value={run.run_id}>
                {run.run_id}
              </option>
            ))}
          </select>
        </label>
      </header>

      {matrixStatus === "loading" ? <EmptyState title="Loading matrix" detail="Reading existing quality_matrix artifacts." /> : null}
      {matrixStatus === "browser-preview" ? <EmptyState title="Desktop runtime required" detail="Matrix loading requires Tauri IPC." /> : null}
      {matrixStatus === "failed" ? <EmptyState title="Matrix failed" detail={matrixError ?? "load_quality_matrix returned an error."} /> : null}

      {matrixStatus === "ready" || matrixStatus === "empty" ? (
        <>
          <div className="matrix-summary-grid">
            <MatrixSummaryCard label="Total tickers" value={rows.length} />
            <MatrixSummaryCard label="Average quality" value={averageQuality(rows)} />
            <MatrixSummaryCard label="Warnings" value={warningsCount} />
            <MatrixSummaryCard label="Hard failures" value={hardFailures} />
            <MatrixSummaryCard label="Provider failures" value={providerFailures} />
          </div>

          <div className="matrix-legend" aria-label="Quality score legend">
            <span><i className="legend-dot legend-dot--good" />85+</span>
            <span><i className="legend-dot legend-dot--ok" />70-84</span>
            <span><i className="legend-dot legend-dot--weak" />60-69</span>
            <span><i className="legend-dot legend-dot--fail" />&lt;60</span>
            <span><i className="legend-dot legend-dot--unknown" />missing</span>
          </div>

          <div className="matrix-layout">
            <div className="matrix-board" role="grid" aria-label="Ticker quality matrix">
              {rows.length === 0 ? (
                <EmptyState title="No quality rows" detail="This training run has no quality matrix rows." />
              ) : (
                rows.map((row) => (
                  <button
                    aria-label={`${row.ticker} quality ${row.quality_score ?? "unknown"}`}
                    className={`matrix-cell ${scoreClass(row)}${selectedRow?.ticker === row.ticker ? " matrix-cell--selected" : ""}`}
                    key={`${row.ticker}-${row.market ?? "market"}`}
                    onClick={() => onSelectRow(row)}
                    title={`${row.ticker}\nScore: ${row.quality_score ?? "unknown"}\nStatus: ${row.status ?? "unknown"}\nIssues: ${row.issue_types.join(", ") || "none"}\nHard failures: ${row.hard_failures.join(", ") || "none"}`}
                    type="button"
                  >
                    <span>{row.ticker}</span>
                  </button>
                ))
              )}
            </div>

            <aside className="matrix-inspector" aria-label="Regression matrix inspector">
              <section className="matrix-panel-section">
                <span className="subsection-title">Selected ticker</span>
                {selectedRow ? (
                  <>
                    <strong className="matrix-selected-ticker">{selectedRow.ticker}</strong>
                    <dl className="detail-kv-grid detail-kv-grid--wide">
                      <KeyValue label="Score" value={selectedRow.quality_score ?? "unknown"} />
                      <KeyValue label="Grade" value={selectedRow.grade ?? "unknown"} />
                      <KeyValue label="Status" value={selectedRow.status ?? "unknown"} />
                      <KeyValue label="AI source" value={selectedRow.ai_source ?? "unknown"} />
                      <KeyValue label="Provider" value={selectedRow.provider_status ?? "unknown"} />
                    </dl>
                    <div className="split-lists split-lists--two">
                      <div>
                        <span className="subsection-title">Issues</span>
                        <SimpleList emptyLabel="No issue types." items={selectedRow.issue_types} />
                      </div>
                      <div>
                        <span className="subsection-title">Hard failures</span>
                        <SimpleList emptyLabel="No hard failures." items={selectedRow.hard_failures} />
                      </div>
                    </div>
                  </>
                ) : (
                  <p className="muted-copy">Select a ticker cell to inspect quality details.</p>
                )}
              </section>

              <section className="matrix-panel-section">
                <span className="subsection-title">Issue distribution</span>
                {matrix?.issue_distribution.length ? (
                  <ul className="issue-bars">
                    {matrix.issue_distribution.slice(0, 8).map((item) => (
                      <li key={item.issue_type}>
                        <span>{item.issue_type}</span>
                        <strong>{item.count}</strong>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="muted-copy">No issue distribution available.</p>
                )}
              </section>

              {matrix?.warnings.length ? (
                <section className="matrix-panel-section matrix-panel-section--warning">
                  <span className="subsection-title">Matrix warnings</span>
                  <SimpleList emptyLabel="No matrix warnings." items={matrix.warnings} />
                </section>
              ) : null}
            </aside>
          </div>
        </>
      ) : null}
    </section>
  );
}

function KeyValue({ label, value }: { label: string; value: ReactNode }): JSX.Element {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
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
  const [runSearch, setRunSearch] = useState<string>("");
  const [selectedRunKey, setSelectedRunKey] = useState<string | null>(null);
  const [activeRunDetail, setActiveRunDetail] = useState<RunDetail | null>(null);
  const [runDetailStatus, setRunDetailStatus] = useState<RunDetailStatus>("idle");
  const [runDetailError, setRunDetailError] = useState<string | null>(null);
  const [mode, setMode] = useState<StudioMode>("runs");
  const [detailTab, setDetailTab] = useState<RunDetailTab>("summary");
  const [trainingRuns, setTrainingRuns] = useState<TrainingRunSummary[]>([]);
  const [trainingRunsStatus, setTrainingRunsStatus] = useState<TrainingRunsStatus>("idle");
  const [trainingRunsError, setTrainingRunsError] = useState<string | null>(null);
  const [selectedTrainingRunId, setSelectedTrainingRunId] = useState<string | null>(null);
  const [qualityMatrix, setQualityMatrix] = useState<QualityMatrix | null>(null);
  const [matrixStatus, setMatrixStatus] = useState<MatrixStatus>("idle");
  const [matrixError, setMatrixError] = useState<string | null>(null);
  const [selectedMatrixRow, setSelectedMatrixRow] = useState<QualityMatrixRow | null>(null);

  const filteredRuns = useMemo(() => {
    const query = runSearch.trim().toLowerCase();
    if (!query) {
      return runs;
    }
    return runs.filter((run) => `${run.ticker} ${run.run_id} ${run.market ?? ""} ${run.provider ?? ""}`.toLowerCase().includes(query));
  }, [runSearch, runs]);

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
          setAppInfoError(message.includes("__TAURI__") ? "Tauri IPC is unavailable in browser preview." : message);
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
          setRunsError(message.includes("__TAURI__") ? "Real run discovery requires the desktop runtime." : message);
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
    setDetailTab("summary");

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
          setRunDetailError(message.includes("__TAURI__") ? "Real detail loading requires the desktop runtime." : message);
        }
      });

    return () => {
      mounted = false;
    };
  }, [selectedRun]);

  useEffect(() => {
    if (mode !== "matrix" || trainingRunsStatus !== "idle") {
      return;
    }

    let mounted = true;
    setTrainingRunsStatus("loading");
    setTrainingRunsError(null);

    listTrainingRuns()
      .then((loadedRuns) => {
        if (mounted) {
          setTrainingRuns(loadedRuns);
          setTrainingRunsStatus(loadedRuns.length > 0 ? "ready" : "empty");
          setSelectedTrainingRunId((current) => current ?? (loadedRuns[0]?.run_id ?? null));
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          const message = error instanceof Error ? error.message : String(error);
          setTrainingRuns([]);
          setTrainingRunsStatus(message.includes("__TAURI__") ? "browser-preview" : "failed");
          setTrainingRunsError(message.includes("__TAURI__") ? "Regression Matrix requires the desktop runtime." : message);
        }
      });

    return () => {
      mounted = false;
    };
  }, [mode, trainingRunsStatus]);

  useEffect(() => {
    if (mode !== "matrix" || !selectedTrainingRunId) {
      return;
    }

    let mounted = true;
    setMatrixStatus("loading");
    setMatrixError(null);
    setSelectedMatrixRow(null);

    loadQualityMatrix(selectedTrainingRunId)
      .then((matrix) => {
        if (mounted) {
          setQualityMatrix(matrix);
          setMatrixStatus(matrix.rows.length > 0 ? "ready" : "empty");
          setSelectedMatrixRow(matrix.rows[0] ?? null);
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          const message = error instanceof Error ? error.message : String(error);
          setQualityMatrix(null);
          setMatrixStatus(message.includes("__TAURI__") ? "browser-preview" : "failed");
          setMatrixError(message.includes("__TAURI__") ? "Matrix loading requires the desktop runtime." : message);
        }
      });

    return () => {
      mounted = false;
    };
  }, [mode, selectedTrainingRunId]);

  const warnings = collectWarnings(activeRunDetail);
  const dataGapCount = countDataGaps(activeRunDetail);

  return (
    <AppShell>
      <Sidebar
        error={runsError}
        mode={mode}
        runs={filteredRuns}
        runsStatus={runsStatus}
        search={runSearch}
        selectedRunKey={selectedRunKey}
        onChangeMode={setMode}
        onSearch={setRunSearch}
        onSelectRun={(run) => setSelectedRunKey(runKey(run))}
      />

      <section className="workspace" aria-label={mode === "runs" ? "Report Workspace" : "Regression Matrix Workspace"}>
        {mode === "runs" ? (
          <>
            <RunWorkspaceHeader
              detail={activeRunDetail}
              detailStatus={runDetailStatus}
              ipcMessage={ipcMessage}
              selectedRun={selectedRun}
            />
            <PrimaryActionBar detail={activeRunDetail} />
            <DetailTabs
              activeTab={detailTab}
              chartCount={activeRunDetail?.charts.length ?? 0}
              dataGapCount={dataGapCount}
              warningCount={warnings.length}
              onChange={setDetailTab}
            />
            <section className="workspace-scroll" aria-label="Run workspace content">
              <RunDetailPanel activeTab={detailTab} detail={activeRunDetail} error={runDetailError} status={runDetailStatus} />
            </section>
            <DiagnosticsDrawer detail={activeRunDetail} status={runDetailStatus} />
          </>
        ) : (
          <section className="workspace-scroll workspace-scroll--matrix" aria-label="Regression Matrix content">
            <RegressionMatrixHub
              error={trainingRunsError}
              matrix={qualityMatrix}
              matrixError={matrixError}
              matrixStatus={matrixStatus}
              onSelectRow={setSelectedMatrixRow}
              onSelectRun={setSelectedTrainingRunId}
              runs={trainingRuns}
              selectedRow={selectedMatrixRow}
              selectedTrainingRunId={selectedTrainingRunId}
              trainingStatus={trainingRunsStatus}
            />
          </section>
        )}

        {mode === "matrix" ? (
          <aside className="matrix-footer-strip">
            <span>Regression Matrix mode</span>
            <span>No run detail cards shown</span>
            <span>No training or external API calls</span>
          </aside>
        ) : null}
      </section>

      <aside className="studio-system-card" aria-label="App information">
        <AppInfoCard appInfo={appInfo} error={appInfoError} status={appInfoStatus} />
      </aside>
    </AppShell>
  );
}
