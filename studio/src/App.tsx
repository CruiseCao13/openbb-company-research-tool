import { type ReactNode, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppInfoCard } from "./components/AppInfoCard";
import { RunDetailPanel } from "./components/RunDetailPanel";
import {
  getAppInfo,
  listRuns,
  listTrainingRuns,
  loadQualityMatrix,
  loadRunDetail,
  openArtifact
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

type ResearchCardConfig = {
  title: string;
  badge: BadgeVariant;
  body: string;
};

type StudioView = "runs" | "regression";
type TrainingRunsStatus = "idle" | "loading" | "ready" | "empty" | "failed" | "browser-preview";
type MatrixStatus = "idle" | "loading" | "ready" | "empty" | "failed" | "browser-preview";

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

function ViewTabs({ activeView, onChange }: { activeView: StudioView; onChange: (view: StudioView) => void }): JSX.Element {
  return (
    <nav className="view-tabs" aria-label="Studio view">
      <button className={activeView === "runs" ? "view-tab view-tab--active" : "view-tab"} onClick={() => onChange("runs")} type="button">
        Research Runs
      </button>
      <button
        className={activeView === "regression" ? "view-tab view-tab--active" : "view-tab"}
        onClick={() => onChange("regression")}
        type="button"
      >
        Regression Matrix
      </button>
    </nav>
  );
}

function TopStatusStrip({
  detailStatus,
  ipcMessage,
  selectedRun
}: {
  detailStatus: RunDetailStatus;
  ipcMessage: string;
  selectedRun: RunSummary | null;
}): JSX.Element {
  return (
    <header className="top-status-strip" aria-label="Studio status">
      <div className="status-cluster">
        <span className="status-dot" aria-hidden="true" />
        <span>Studio shell ready</span>
      </div>
      <span>No external API used</span>
      <span>{selectedRun ? `${selectedRun.ticker} / ${selectedRun.run_id}` : "No run loaded"}</span>
      <StatusBadge variant={detailStatus === "ready" ? statusToBadge(selectedRun?.status ?? null) : "UNKNOWN"} />
      <span className="ipc-readout">{ipcMessage}</span>
    </header>
  );
}

function booleanLabel(value: boolean | null): string {
  if (value === null) {
    return "unknown";
  }
  return value ? "yes" : "no";
}

function compactList(items: string[], limit = 4): { visible: string[]; remaining: number } {
  return {
    visible: items.slice(0, limit),
    remaining: Math.max(0, items.length - limit)
  };
}

function ProvenanceList({ emptyLabel, items }: { emptyLabel: string; items: string[] }): JSX.Element {
  const { remaining, visible } = compactList(items);
  if (visible.length === 0) {
    return <p>{emptyLabel}</p>;
  }

  return (
    <ul className="bottom-compact-list">
      {visible.map((item) => (
        <li key={item}>{item}</li>
      ))}
      {remaining > 0 ? <li>{remaining} more</li> : null}
    </ul>
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

function provenanceWarnings(detail: RunDetail): string[] {
  const warnings = new Set<string>([
    ...detail.blueprint.data_gaps,
    ...detail.provider.missing_fields,
    ...detail.warnings
  ]);

  if (detail.status.human_review_required || detail.self_review.human_review_required) {
    warnings.add("Human review required");
  }
  if (detail.ai_usage.local_mock_used) {
    warnings.add("Local/mock AI output is not external AI proof");
  }
  if (detail.provider.mock) {
    warnings.add("Provider metadata says mock data was used");
  }
  for (const value of [
    detail.status.overall_status,
    detail.status.provider_status,
    detail.status.visual_lint_status,
    detail.self_review.framework_fit_check,
    detail.self_review.numeric_consistency_check,
    detail.self_review.money_flow_check
  ]) {
    const normalized = value?.toLowerCase() ?? "";
    if (normalized.includes("unsupported")) {
      warnings.add(value ?? "Unsupported claim warning");
    }
  }

  return Array.from(warnings);
}

function BottomProvenanceBar({
  detail,
  status
}: {
  detail: RunDetail | null;
  status: RunDetailStatus;
}): JSX.Element {
  const [message, setMessage] = useState<string>("No provenance artifact action yet.");
  const isReady = status === "ready" && detail !== null;
  const warnings = detail ? provenanceWarnings(detail) : [];
  const badge: BadgeVariant = !isReady
    ? "DATA_GAP"
    : detail.provider.mock
      ? "FAIL"
      : detail.status.human_review_required || detail.self_review.human_review_required || warnings.length > 0
        ? "WARNING"
        : detail.ai_usage.external_ai_used
          ? "EXTERNAL_AI"
          : detail.ai_usage.local_mock_used
            ? "LOCAL_MOCK"
            : "UNKNOWN";

  async function handleOpen(label: string, path: string | null | undefined): Promise<void> {
    if (!path) {
      return;
    }
    setMessage(`Opening ${label}...`);
    try {
      const result = await openArtifact(path);
      setMessage(`${result.message}: ${result.path}`);
    } catch (error: unknown) {
      const text = error instanceof Error ? error.message : String(error);
      const browserPreview = text.includes("__TAURI__")
        ? "Tauri IPC is unavailable in browser preview. Artifact opening requires the desktop runtime."
        : text;
      setMessage(`Artifact action failed: ${browserPreview}`);
    }
  }

  return (
    <section className="bottom-bar" aria-label="Provenance and data gaps">
      <div className="bottom-bar__header">
        <h3>Provenance &amp; Data Gaps</h3>
        <div className="bottom-badge-row">
          <StatusBadge variant={badge} />
          {detail?.ai_usage.external_ai_used ? <StatusBadge variant="EXTERNAL_AI" /> : null}
          {detail?.ai_usage.local_mock_used ? <StatusBadge variant="LOCAL_MOCK" /> : null}
          {detail?.ai_usage.cache_hits ? <StatusBadge variant="CACHE" /> : null}
          {detail?.status.human_review_required || detail?.self_review.human_review_required ? (
            <StatusBadge variant="HUMAN_REVIEW" />
          ) : null}
          {detail?.provider.mock ? <StatusBadge variant="PROVIDER_MOCK" /> : null}
        </div>
      </div>

      {!isReady ? (
        <div className="bottom-bar__empty">
          <strong>{status === "loading" ? "Loading run provenance..." : "Select a run to inspect AI provenance and data gaps."}</strong>
          <span>No external API, provider, or filesystem read is triggered from this panel.</span>
        </div>
      ) : (
        <div className="bottom-bar__grid">
          <div className="provenance-cell">
            <span>AI provenance</span>
            <dl className="bottom-kv">
              <div>
                <dt>Source</dt>
                <dd>{detail.ai_usage.source ?? "UNKNOWN"}</dd>
              </div>
              <div>
                <dt>External</dt>
                <dd>{booleanLabel(detail.ai_usage.external_ai_used)}</dd>
              </div>
              <div>
                <dt>Local mock</dt>
                <dd>{booleanLabel(detail.ai_usage.local_mock_used)}</dd>
              </div>
              <div>
                <dt>New calls</dt>
                <dd>{detail.ai_usage.new_external_ai_calls ?? "unknown"}</dd>
              </div>
              <div>
                <dt>Cache hits</dt>
                <dd>{detail.ai_usage.cache_hits ?? "unknown"}</dd>
              </div>
              <div>
                <dt>Model</dt>
                <dd>{detail.ai_usage.model ?? "unknown"}</dd>
              </div>
            </dl>
            <ProvenanceList emptyLabel="No prompt versions reported." items={detail.ai_usage.prompt_versions} />
            <button
              className="bottom-action"
              disabled={!detail.artifacts.ai_usage_path}
              onClick={() => void handleOpen("AI usage", detail.artifacts.ai_usage_path)}
              type="button"
            >
              Open AI Usage
            </button>
          </div>

          <div className="provenance-cell">
            <span>Provider source</span>
            <dl className="bottom-kv">
              <div>
                <dt>Provider</dt>
                <dd>{detail.provider.provider ?? "UNKNOWN"}</dd>
              </div>
              <div>
                <dt>Source</dt>
                <dd>{detail.provider.source ?? "unknown"}</dd>
              </div>
              <div>
                <dt>Adapter</dt>
                <dd>{detail.provider.provider_adapter ?? "unknown"}</dd>
              </div>
              <div>
                <dt>Package</dt>
                <dd>{booleanLabel(detail.provider.package_used)}</dd>
              </div>
              <div>
                <dt>Mock</dt>
                <dd>{booleanLabel(detail.provider.mock)}</dd>
              </div>
              <div>
                <dt>Market</dt>
                <dd>{detail.provider.market ?? "unknown"} / {detail.provider.currency ?? "unknown"}</dd>
              </div>
            </dl>
            <ProvenanceList emptyLabel="No provider limitations reported." items={detail.provider.limitations} />
          </div>
          <div className="provenance-cell">
            <span>Data gaps / warnings</span>
            <ProvenanceList emptyLabel="No data gaps, missing fields, or warnings reported." items={warnings} />
            <button
              className="bottom-action"
              disabled={!detail.artifacts.blueprint_path}
              onClick={() => void handleOpen("research blueprint", detail.artifacts.blueprint_path)}
              type="button"
            >
              Open Blueprint
            </button>
            <p className="bottom-message">{message}</p>
          </div>
        </div>
      )}
    </section>
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
  const average = scores.reduce((sum, score) => sum + score, 0) / scores.length;
  return average.toFixed(1);
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
  onSelectRun,
  onSelectRow,
  runs,
  selectedRow,
  selectedTrainingRunId,
  trainingStatus
}: {
  error: string | null;
  matrix: QualityMatrix | null;
  matrixError: string | null;
  matrixStatus: MatrixStatus;
  onSelectRun: (runId: string) => void;
  onSelectRow: (row: QualityMatrixRow) => void;
  runs: TrainingRunSummary[];
  selectedRow: QualityMatrixRow | null;
  selectedTrainingRunId: string | null;
  trainingStatus: TrainingRunsStatus;
}): JSX.Element {
  if (trainingStatus === "loading") {
    return <EmptyState title="Loading training runs..." detail="Scanning reports/training_runs through Tauri IPC." />;
  }

  if (trainingStatus === "browser-preview") {
    return <EmptyState title="Regression Matrix needs Tauri" detail="Browser preview cannot access training run artifacts." />;
  }

  if (trainingStatus === "failed") {
    return <EmptyState title="Training run discovery failed" detail={error ?? "The list_training_runs command returned an error."} />;
  }

  if (runs.length === 0) {
    return <EmptyState title="No training runs found" detail="No reports/training_runs folders were discovered." />;
  }

  const rows = matrix?.rows ?? [];
  const hardFailures = rows.reduce((count, row) => count + row.hard_failures.length, 0);
  const providerFailures = countProviderFailures(rows);
  const warningsCount = (matrix?.warnings.length ?? 0) + rows.filter((row) => row.issue_types.length > 0).length;

  return (
    <section className="matrix-hub" aria-label="Regression Matrix Hub">
      <div className="matrix-toolbar">
        <div>
          <p className="eyebrow">Quality control board</p>
          <h3>Regression Matrix</h3>
        </div>
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
      </div>

      {matrixStatus === "loading" ? <EmptyState title="Loading matrix..." detail="Reading existing quality_matrix artifacts." /> : null}
      {matrixStatus === "browser-preview" ? (
        <EmptyState title="Matrix loading needs Tauri" detail="Browser preview cannot read training run artifacts." />
      ) : null}
      {matrixStatus === "failed" ? <EmptyState title="Matrix failed" detail={matrixError ?? "The load_quality_matrix command returned an error."} /> : null}

      {matrixStatus === "ready" || matrixStatus === "empty" ? (
        <>
          <div className="matrix-summary-grid">
            <MatrixSummaryCard label="Tickers" value={rows.length} />
            <MatrixSummaryCard label="Avg quality" value={averageQuality(rows)} />
            <MatrixSummaryCard label="Warnings" value={warningsCount} />
            <MatrixSummaryCard label="Hard failures" value={hardFailures} />
            <MatrixSummaryCard label="Provider failures" value={providerFailures} />
          </div>

          {matrix?.warnings.length ? (
            <div className="matrix-warning-strip">
              {matrix.warnings.slice(0, 3).map((warning) => (
                <span key={warning}>{warning}</span>
              ))}
            </div>
          ) : null}

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

            <aside className="matrix-side-panel" aria-label="Regression matrix detail">
              <div className="matrix-panel-section">
                <span className="subsection-title">Issue distribution</span>
                {matrix?.issue_distribution.length ? (
                  <ul className="compact-list">
                    {matrix.issue_distribution.slice(0, 8).map((item) => (
                      <li key={item.issue_type}>
                        {item.issue_type}: {item.count}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="muted-copy">No issue distribution available.</p>
                )}
              </div>

              <div className="matrix-panel-section">
                <span className="subsection-title">Selected ticker</span>
                {selectedRow ? (
                  <dl className="detail-kv-grid detail-kv-grid--wide">
                    <KeyValueLike label="Ticker" value={selectedRow.ticker} />
                    <KeyValueLike label="Score" value={selectedRow.quality_score ?? "unknown"} />
                    <KeyValueLike label="Grade" value={selectedRow.grade ?? "unknown"} />
                    <KeyValueLike label="Status" value={selectedRow.status ?? "unknown"} />
                    <KeyValueLike label="AI source" value={selectedRow.ai_source ?? "unknown"} />
                    <KeyValueLike label="Provider" value={selectedRow.provider_status ?? "unknown"} />
                  </dl>
                ) : (
                  <p className="muted-copy">Select a ticker cell to inspect quality details.</p>
                )}
                {selectedRow ? (
                  <div className="split-lists">
                    <div>
                      <span className="subsection-title">Issues</span>
                      <ProvenanceList emptyLabel="No issue types." items={selectedRow.issue_types} />
                    </div>
                    <div>
                      <span className="subsection-title">Hard failures</span>
                      <ProvenanceList emptyLabel="No hard failures." items={selectedRow.hard_failures} />
                    </div>
                  </div>
                ) : null}
              </div>
            </aside>
          </div>
        </>
      ) : null}
    </section>
  );
}

function KeyValueLike({ label, value }: { label: string; value: ReactNode }): JSX.Element {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
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
  const [activeView, setActiveView] = useState<StudioView>("runs");
  const [trainingRuns, setTrainingRuns] = useState<TrainingRunSummary[]>([]);
  const [trainingRunsStatus, setTrainingRunsStatus] = useState<TrainingRunsStatus>("idle");
  const [trainingRunsError, setTrainingRunsError] = useState<string | null>(null);
  const [selectedTrainingRunId, setSelectedTrainingRunId] = useState<string | null>(null);
  const [qualityMatrix, setQualityMatrix] = useState<QualityMatrix | null>(null);
  const [matrixStatus, setMatrixStatus] = useState<MatrixStatus>("idle");
  const [matrixError, setMatrixError] = useState<string | null>(null);
  const [selectedMatrixRow, setSelectedMatrixRow] = useState<QualityMatrixRow | null>(null);

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

  useEffect(() => {
    if (activeView !== "regression" || trainingRunsStatus !== "idle") {
      return;
    }

    let mounted = true;
    setTrainingRunsStatus("loading");
    setTrainingRunsError(null);

    listTrainingRuns()
      .then((runs) => {
        if (mounted) {
          setTrainingRuns(runs);
          setTrainingRunsStatus(runs.length > 0 ? "ready" : "empty");
          setSelectedTrainingRunId((current) => current ?? (runs[0]?.run_id ?? null));
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          const message = error instanceof Error ? error.message : String(error);
          setTrainingRuns([]);
          setTrainingRunsStatus(message.includes("__TAURI__") ? "browser-preview" : "failed");
          setTrainingRunsError(
            message.includes("__TAURI__")
              ? "Tauri IPC is unavailable in browser preview. Regression Matrix requires the desktop runtime."
              : message
          );
        }
      });

    return () => {
      mounted = false;
    };
  }, [activeView, trainingRunsStatus]);

  useEffect(() => {
    if (activeView !== "regression" || !selectedTrainingRunId) {
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
          setMatrixError(
            message.includes("__TAURI__")
              ? "Tauri IPC is unavailable in browser preview. Matrix loading requires the desktop runtime."
              : message
          );
        }
      });

    return () => {
      mounted = false;
    };
  }, [activeView, selectedTrainingRunId]);

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
        <TopStatusStrip detailStatus={runDetailStatus} ipcMessage={ipcMessage} selectedRun={selectedRun} />

        <header className="workspace-header">
          <div>
            <p className="eyebrow">Run intelligence workspace</p>
            <h2>Research Run Detail</h2>
            <p>Select a run to inspect locked data, AI provenance, validator logs, and report artifacts.</p>
          </div>
          <ViewTabs activeView={activeView} onChange={setActiveView} />
        </header>

        {activeView === "runs" ? (
          <section className="card-grid" aria-label="Research detail cards">
            <AppInfoCard appInfo={appInfo} error={appInfoError} status={appInfoStatus} />
            <RunDetailPanel detail={activeRunDetail} error={runDetailError} status={runDetailStatus} />
            {runDetailStatus === "idle" || runDetailStatus === "loading"
              ? placeholderCards.map((card, index) => <ResearchCard card={card} key={card.title} wide={index >= 3} />)
              : null}
          </section>
        ) : (
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
        )}

        <BottomProvenanceBar detail={activeRunDetail} status={runDetailStatus} />
      </section>
    </AppShell>
  );
}
