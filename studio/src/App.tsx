import { type CSSProperties, type PointerEvent, type ReactNode, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { AppInfoCard } from "./components/AppInfoCard";
import { RunDetailPanel, type RunDetailTab } from "./components/RunDetailPanel";
import i18n, { type StudioLanguage } from "./i18n";
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

type StudioMode = "landing" | "runs" | "matrix";
type StudioDensity = "compact" | "comfortable";
type StudioMotion = "on" | "off";
type StudioGlass = "low" | "medium" | "high";
type MatrixFilter = "ALL" | "PASS" | "WARNING" | "FAIL" | "DATA_GAP" | "LOCAL" | "EXTERNAL";
type TrainingRunsStatus = "idle" | "loading" | "ready" | "empty" | "failed" | "browser-preview";
type MatrixStatus = "idle" | "loading" | "ready" | "empty" | "failed" | "browser-preview";

const detailTabs: Array<{ id: RunDetailTab; labelKey: string }> = [
  { id: "summary", labelKey: "summary" },
  { id: "charts", labelKey: "charts" },
  { id: "audit", labelKey: "auditTrail" },
  { id: "gaps", labelKey: "dataGaps" },
  { id: "artifacts", labelKey: "artifacts" }
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

function CompanyMonogram({
  market,
  status,
  ticker
}: {
  market: string | null | undefined;
  status: string | null | undefined;
  ticker: string | null | undefined;
}): JSX.Element {
  return (
    <div className={`company-monogram company-monogram--${statusToBadge(status ?? null).toLowerCase()}`} aria-hidden="true">
      <strong>{tickerMonogram(ticker)}</strong>
      <span>{market ?? "MKT"}</span>
    </div>
  );
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

function StudioTopBar({
  density,
  language,
  mode,
  motion,
  onOpenSettings,
  onQuickSearch,
  onSetDensity,
  onSetLanguage,
  quickSearch
}: {
  density: StudioDensity;
  language: StudioLanguage;
  mode: StudioMode;
  motion: StudioMotion;
  onOpenSettings: () => void;
  onQuickSearch: (value: string) => void;
  onSetDensity: (density: StudioDensity) => void;
  onSetLanguage: (language: StudioLanguage) => void;
  quickSearch: string;
}): JSX.Element {
  const { t } = useTranslation();

  return (
    <header className="studio-topbar" aria-label="Studio controls">
      <div className="topbar-mode">
        <span>{t("currentMode")}</span>
        <strong>{mode === "landing" ? "Landing" : mode === "matrix" ? t("matrix") : t("runs")}</strong>
      </div>
      <label className="quick-search">
        <span>{t("quickSearch")}</span>
        <input
          onChange={(event) => onQuickSearch(event.target.value)}
          placeholder="AAPL / RKLB / 600519"
          type="search"
          value={quickSearch}
        />
      </label>
      <div className="topbar-controls">
        <button
          className={language === "en" ? "topbar-chip topbar-chip--active" : "topbar-chip"}
          onClick={() => onSetLanguage("en")}
          type="button"
        >
          EN
        </button>
        <button
          className={language === "zh" ? "topbar-chip topbar-chip--active" : "topbar-chip"}
          onClick={() => onSetLanguage("zh")}
          type="button"
        >
          中文
        </button>
        <button
          className="topbar-chip"
          onClick={() => onSetDensity(density === "compact" ? "comfortable" : "compact")}
          type="button"
        >
          {density === "compact" ? t("compact") : t("comfortable")}
        </button>
        <span className="topbar-chip topbar-chip--readout">{motion === "on" ? t("motion") : `${t("motion")} ${t("off")}`}</span>
        <button className="topbar-chip topbar-chip--settings" onClick={onOpenSettings} type="button">
          {t("settings")}
        </button>
      </div>
    </header>
  );
}

function LandingHero({
  onEnter,
  onOpenLatest,
  onOpenMatrix
}: {
  onEnter: () => void;
  onOpenLatest: () => void;
  onOpenMatrix: () => void;
}): JSX.Element {
  const { t } = useTranslation();

  return (
    <section className="landing-hero" aria-label="Studio landing">
      <div className="hero-orbit" aria-hidden="true">
        <span />
        <span />
        <span />
      </div>
      <div className="landing-hero__content">
        <p className="eyebrow">v6 Tauri Research Studio</p>
        <h2>{t("heroTitle")}</h2>
        <p>{t("heroSubtitle")}</p>
        <div className="landing-hero__actions">
          <button className="hero-cta hero-cta--primary" onClick={onEnter} type="button">
            {t("enterStudio")}
          </button>
          <button className="hero-cta" onClick={onOpenLatest} type="button">
            {t("openLatestRun")}
          </button>
          <button className="hero-cta" onClick={onOpenMatrix} type="button">
            View Quality Matrix
          </button>
        </div>
      </div>
      <div className="landing-hero__terminal" aria-label="Research terminal preview">
        <div><span>01</span><strong>Trace the business model</strong></div>
        <div><span>02</span><strong>Verify locked numbers</strong></div>
        <div><span>03</span><strong>Follow the cash conversion</strong></div>
        <div><span>04</span><strong>Inspect provenance and gaps</strong></div>
      </div>
    </section>
  );
}

function SettingsCenter({
  defaultLanding,
  density,
  fontScale,
  glass,
  language,
  motion,
  onClose,
  onDefaultLanding,
  onDensity,
  onFontScale,
  onGlass,
  onLanguage,
  onMatrixDefault,
  onMotion,
  onStartLatest,
  onWarningsFirst,
  openMatrixByDefault,
  open,
  startOnLatest,
  warningsFirst
}: {
  defaultLanding: boolean;
  density: StudioDensity;
  fontScale: number;
  glass: StudioGlass;
  language: StudioLanguage;
  motion: StudioMotion;
  onClose: () => void;
  onDefaultLanding: (value: boolean) => void;
  onDensity: (value: StudioDensity) => void;
  onFontScale: (value: number) => void;
  onGlass: (value: StudioGlass) => void;
  onLanguage: (value: StudioLanguage) => void;
  onMatrixDefault: (value: boolean) => void;
  onMotion: (value: StudioMotion) => void;
  onStartLatest: (value: boolean) => void;
  onWarningsFirst: (value: boolean) => void;
  openMatrixByDefault: boolean;
  open: boolean;
  startOnLatest: boolean;
  warningsFirst: boolean;
}): JSX.Element | null {
  const { t } = useTranslation();
  if (!open) {
    return null;
  }

  return (
    <div className="settings-backdrop" role="presentation">
      <aside className="settings-center" aria-label="Settings center">
        <div className="settings-center__header">
          <div>
            <p className="eyebrow">Studio Control</p>
            <h2>{t("settings")}</h2>
          </div>
          <button className="topbar-chip" onClick={onClose} type="button">
            Close
          </button>
        </div>
        <div className="settings-grid">
          <SettingGroup title={t("language")}>
            <SegmentedChoice
              options={[
                ["en", "English"],
                ["zh", "中文"]
              ]}
              value={language}
              onChange={(value) => onLanguage(value as StudioLanguage)}
            />
          </SettingGroup>
          <SettingGroup title={t("density")}>
            <SegmentedChoice
              options={[
                ["compact", t("compact")],
                ["comfortable", t("comfortable")]
              ]}
              value={density}
              onChange={(value) => onDensity(value as StudioDensity)}
            />
          </SettingGroup>
          <SettingGroup title={t("motion")}>
            <SegmentedChoice
              options={[
                ["on", t("on")],
                ["off", t("off")]
              ]}
              value={motion}
              onChange={(value) => onMotion(value as StudioMotion)}
            />
          </SettingGroup>
          <SettingGroup title={t("glass")}>
            <SegmentedChoice
              options={[
                ["low", t("low")],
                ["medium", t("medium")],
                ["high", t("high")]
              ]}
              value={glass}
              onChange={(value) => onGlass(value as StudioGlass)}
            />
          </SettingGroup>
          <SettingGroup title={t("fontScale")}>
            <input
              max="1.12"
              min="0.92"
              onChange={(event) => onFontScale(Number(event.target.value))}
              step="0.02"
              type="range"
              value={fontScale}
            />
          </SettingGroup>
          <SettingGroup title={t("warningsFirst")}>
            <label className="settings-toggle">
              <input checked={warningsFirst} onChange={(event) => onWarningsFirst(event.target.checked)} type="checkbox" />
              <span>{warningsFirst ? t("on") : t("off")}</span>
            </label>
          </SettingGroup>
          <SettingGroup title={t("defaultLanding")}>
            <label className="settings-toggle">
              <input checked={defaultLanding} onChange={(event) => onDefaultLanding(event.target.checked)} type="checkbox" />
              <span>{defaultLanding ? t("on") : t("off")}</span>
            </label>
          </SettingGroup>
          <SettingGroup title="Start on latest run">
            <label className="settings-toggle">
              <input checked={startOnLatest} onChange={(event) => onStartLatest(event.target.checked)} type="checkbox" />
              <span>{startOnLatest ? t("on") : t("off")}</span>
            </label>
          </SettingGroup>
          <SettingGroup title="Open matrix by default">
            <label className="settings-toggle">
              <input checked={openMatrixByDefault} onChange={(event) => onMatrixDefault(event.target.checked)} type="checkbox" />
              <span>{openMatrixByDefault ? t("on") : t("off")}</span>
            </label>
          </SettingGroup>
        </div>
      </aside>
    </div>
  );
}

function SettingGroup({ children, title }: { children: ReactNode; title: string }): JSX.Element {
  return (
    <section className="setting-group">
      <span>{title}</span>
      {children}
    </section>
  );
}

function SegmentedChoice({
  onChange,
  options,
  value
}: {
  onChange: (value: string) => void;
  options: Array<[string, string]>;
  value: string;
}): JSX.Element {
  return (
    <div className="segmented-choice">
      {options.map(([optionValue, label]) => (
        <button
          className={value === optionValue ? "segmented-choice__button segmented-choice__button--active" : "segmented-choice__button"}
          key={optionValue}
          onClick={() => onChange(optionValue)}
          type="button"
        >
          {label}
        </button>
      ))}
    </div>
  );
}

function Sidebar({
  error,
  mode,
  onChangeMode,
  onOpenSettings,
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
  onOpenSettings: () => void;
  onSearch: (search: string) => void;
  onSelectRun: (run: RunSummary) => void;
  runs: RunSummary[];
  runsStatus: RunsStatus;
  search: string;
  selectedRunKey: string | null;
}): JSX.Element {
  const { t } = useTranslation();
  return (
    <aside className="nav-rail" aria-label="Studio navigation">
      <div className="brand-block">
        <p className="eyebrow">v6 desktop studio</p>
        <h1>{t("studioTitle")}</h1>
        <span>{t("browseRuns")}</span>
      </div>

      <div className="mode-switch" role="tablist" aria-label="Workspace mode">
        <button
          className={mode === "runs" ? "mode-switch__button mode-switch__button--active" : "mode-switch__button"}
          onClick={() => onChangeMode("runs")}
          type="button"
        >
          {t("runs")}
        </button>
        <button
          className={mode === "matrix" ? "mode-switch__button mode-switch__button--active" : "mode-switch__button"}
          onClick={() => onChangeMode("matrix")}
          type="button"
        >
          {t("matrix")}
        </button>
        <button className="mode-switch__button" onClick={onOpenSettings} type="button">
          {t("settings")}
        </button>
      </div>

      <label className="run-search">
        <span>{t("filterRuns")}</span>
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
              <span className="run-list-item__identity">
                <CompanyMonogram market={run.market} status={run.status} ticker={run.ticker} />
                <strong>{run.ticker}</strong>
              </span>
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
  const { t } = useTranslation();
  const [message, setMessage] = useState<string>(t("chooseArtifact"));
  const [busyLabel, setBusyLabel] = useState<string | null>(null);

  const actions = [
    { label: t("openReport"), path: detail?.artifacts.markdown_report_path ?? null, action: "open" as const },
    { label: t("openDashboard"), path: detail?.artifacts.dashboard_path ?? null, action: "open" as const },
    { label: t("openPdf"), path: detail?.artifacts.pdf_report_path ?? null, action: "open" as const },
    { label: t("revealFolder"), path: detail?.run_folder ?? null, action: "reveal" as const },
    { label: t("openAiUsage"), path: detail?.artifacts.ai_usage_path ?? null, action: "open" as const },
    { label: t("openValidator"), path: detail?.artifacts.validator_report_path ?? null, action: "open" as const },
    { label: t("openProvider"), path: detail?.artifacts.provider_payload_path ?? null, action: "open" as const }
  ];

  async function handleAction(label: string, path: string | null, action: "open" | "reveal"): Promise<void> {
    if (!path) {
      return;
    }
    setBusyLabel(label);
    setMessage(`${action === "open" ? "Opening" : "Revealing"} ${label}...`);
    try {
      const result = action === "open" ? await openArtifact(path) : await revealInFolder(path);
      setMessage(result.ok ? `${label} ready.` : `${label} action returned a warning.`);
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
            title={action.path ? `${action.label} is available` : `${action.label} unavailable for this run`}
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
  const { t } = useTranslation();
  const warnings = collectWarnings(detail);
  const chartCount = detail?.charts.length ?? 0;
  const artifacts = artifactCount(detail);

  return (
    <header className="workspace-hero">
      <div className="workspace-hero__identity">
        <p className="eyebrow">{t("reportWorkspace")}</p>
        <div className="workspace-hero__title-row">
          <CompanyMonogram
            market={detail?.provider.market ?? selectedRun?.market}
            status={detail?.status.overall_status ?? selectedRun?.status}
            ticker={selectedRun?.ticker}
          />
          <h2>{selectedRun?.ticker ?? t("noRunSelected")}</h2>
          {detail?.company.name ? <span>{detail.company.name}</span> : null}
        </div>
        <p className="mono-path">{selectedRun ? selectedRun.run_id : t("selectRun")}</p>
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
  const { t } = useTranslation();
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
          <span>{t(tab.labelKey)}</span>
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
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState<boolean>(false);
  const warnings = collectWarnings(detail);
  const dataGapCount = countDataGaps(detail);
  const isReady = status === "ready" && detail !== null;

  return (
    <aside className={`diagnostics-drawer${expanded ? " diagnostics-drawer--expanded" : ""}`} aria-label="Diagnostics drawer">
      <button className="diagnostics-strip" onClick={() => setExpanded((current) => !current)} type="button">
        <span>{t("diagnostics")}</span>
        <StatusBadge variant={detail?.ai_usage.external_ai_used ? "EXTERNAL_AI" : detail?.ai_usage.local_mock_used ? "LOCAL_MOCK" : "UNKNOWN"} />
        <span>{detail?.provider.provider ?? "provider unknown"}</span>
        <span>warnings {warnings.length}</span>
        <span>data gaps {dataGapCount}</span>
        {detail?.status.human_review_required || detail?.self_review.human_review_required ? (
          <StatusBadge variant="HUMAN_REVIEW" />
        ) : null}
        <small>{expanded ? t("collapse") : t("expand")}</small>
      </button>
      {expanded ? (
        <div className="diagnostics-panel">
          {!isReady ? (
            <EmptyState title={status === "loading" ? "Loading diagnostics" : "No run selected"} detail="Select a run to inspect provenance and data gaps." />
          ) : (
            <>
              <section>
                <h3>{t("aiProvenance")}</h3>
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
                <h3>{t("provider")}</h3>
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
                <h3>{t("dataGaps")} & {t("warnings")}</h3>
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
  const { t } = useTranslation();
  const [activeFilter, setActiveFilter] = useState<MatrixFilter>("ALL");
  const rows = useMemo(() => {
    const allRows = matrix?.rows ?? [];
    if (activeFilter === "ALL") {
      return allRows;
    }
    return allRows.filter((row) => {
      const status = `${row.status ?? ""} ${row.issue_types.join(" ")} ${row.hard_failures.join(" ")}`.toUpperCase();
      if (activeFilter === "LOCAL") {
        return (row.ai_source ?? "").toLowerCase().includes("local");
      }
      if (activeFilter === "EXTERNAL") {
        return (row.ai_source ?? "").toLowerCase().includes("external");
      }
      return status.includes(activeFilter);
    });
  }, [activeFilter, matrix?.rows]);
  const totalRows = matrix?.rows ?? [];
  const hardFailures = rows.reduce((count, row) => count + row.hard_failures.length, 0);
  const providerFailures = countProviderFailures(rows);
  const warningsCount = (matrix?.warnings.length ?? 0) + rows.filter((row) => row.issue_types.length > 0).length;

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

  return (
    <section className="matrix-workspace" aria-label="Regression Matrix Hub">
      <header className="matrix-hero">
        <div>
          <p className="eyebrow">Quality control board</p>
          <h2>{t("regressionHub")}</h2>
          <p>Read existing training run quality matrices. No training or external API calls are performed.</p>
        </div>
        <label className="training-selector">
          <span>{t("trainingRun")}</span>
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
            <MatrixSummaryCard label="Total tickers" value={`${rows.length}/${totalRows.length}`} />
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

          <div className="matrix-filter-chips" aria-label="Matrix filters">
            {(["ALL", "PASS", "WARNING", "FAIL", "DATA_GAP", "LOCAL", "EXTERNAL"] as MatrixFilter[]).map((filter) => (
              <button
                className={activeFilter === filter ? "matrix-filter-chip matrix-filter-chip--active" : "matrix-filter-chip"}
                key={filter}
                onClick={() => setActiveFilter(filter)}
                type="button"
              >
                {filter}
              </button>
            ))}
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

function InsightRail({ detail, warningsFirst }: { detail: RunDetail | null; warningsFirst: boolean }): JSX.Element {
  const { t } = useTranslation();
  const warnings = collectWarnings(detail);
  const warningBlock = (
    <section className="insight-card insight-card--warning">
      <div className="card-header">
        <span className="card-label">{t("warnings")}</span>
        <StatusBadge variant={warnings.length > 0 ? "WARNING" : "PASS"} />
      </div>
      <SimpleList emptyLabel="No warnings for the selected run." items={warnings} />
    </section>
  );
  const cards = [
    <section className="insight-card" key="identity">
      <span className="card-label">{t("companyIdentity")}</span>
      <strong>{detail?.company.name ?? detail?.ticker ?? "No run selected"}</strong>
      <p>{detail?.company.frame ?? "Select a run to inspect company identity."}</p>
    </section>,
    <section className="insight-card" key="money">
      <span className="card-label">{t("moneyFlow")}</span>
      <p>{detail?.financial_interpretation.where_money_comes_from ?? "Money source mechanism will appear here."}</p>
      <p>{detail?.financial_interpretation.where_money_goes ?? "Cash-use mechanism will appear here."}</p>
    </section>,
    warningBlock,
    <section className="insight-card" key="blueprint">
      <span className="card-label">{t("blueprint")}</span>
      <p>{detail?.blueprint.core_thesis ?? "Research blueprint summary will appear after loading a run."}</p>
      <SimpleList emptyLabel="No next checks loaded." items={detail?.blueprint.next_checks ?? []} />
    </section>
  ];

  return <aside className="insight-rail" aria-label="Run insights">{warningsFirst ? [warningBlock, ...cards.filter((card) => card !== warningBlock)] : cards}</aside>;
}

function AppShell({
  children,
  density,
  fontScale,
  glass,
  motion
}: {
  children: ReactNode;
  density: StudioDensity;
  fontScale: number;
  glass: StudioGlass;
  motion: StudioMotion;
}): JSX.Element {
  function handlePointerMove(event: PointerEvent<HTMLElement>): void {
    if (motion === "off") {
      return;
    }
    event.currentTarget.style.setProperty("--pointer-x", `${event.clientX}px`);
    event.currentTarget.style.setProperty("--pointer-y", `${event.clientY}px`);
  }

  return (
    <main
      className="studio-shell"
      data-density={density}
      data-glass={glass}
      data-motion={motion}
      onPointerMove={handlePointerMove}
      style={{ "--studio-font-scale": fontScale } as CSSProperties}
    >
      {children}
    </main>
  );
}

export function App(): JSX.Element {
  const { t } = useTranslation();
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
  const [mode, setMode] = useState<StudioMode>("landing");
  const [detailTab, setDetailTab] = useState<RunDetailTab>("charts");
  const [language, setLanguage] = useState<StudioLanguage>("en");
  const [density, setDensity] = useState<StudioDensity>("comfortable");
  const [motion, setMotion] = useState<StudioMotion>("on");
  const [glass, setGlass] = useState<StudioGlass>("high");
  const [fontScale, setFontScale] = useState<number>(1);
  const [warningsFirst, setWarningsFirst] = useState<boolean>(true);
  const [defaultLanding, setDefaultLanding] = useState<boolean>(true);
  const [startOnLatest, setStartOnLatest] = useState<boolean>(false);
  const [openMatrixByDefault, setOpenMatrixByDefault] = useState<boolean>(false);
  const [settingsOpen, setSettingsOpen] = useState<boolean>(false);
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
    const saved = window.localStorage.getItem("v6-studio-ui-settings");
    if (!saved) {
      return;
    }
    try {
      const parsed = JSON.parse(saved) as Partial<{
        defaultLanding: boolean;
        density: StudioDensity;
        fontScale: number;
        glass: StudioGlass;
        language: StudioLanguage;
        motion: StudioMotion;
        openMatrixByDefault: boolean;
        startOnLatest: boolean;
        warningsFirst: boolean;
      }>;
      if (parsed.language === "en" || parsed.language === "zh") setLanguage(parsed.language);
      if (parsed.density === "compact" || parsed.density === "comfortable") setDensity(parsed.density);
      if (parsed.motion === "on" || parsed.motion === "off") setMotion(parsed.motion);
      if (parsed.glass === "low" || parsed.glass === "medium" || parsed.glass === "high") setGlass(parsed.glass);
      if (typeof parsed.fontScale === "number") setFontScale(parsed.fontScale);
      if (typeof parsed.warningsFirst === "boolean") setWarningsFirst(parsed.warningsFirst);
      if (typeof parsed.defaultLanding === "boolean") setDefaultLanding(parsed.defaultLanding);
      if (typeof parsed.startOnLatest === "boolean") setStartOnLatest(parsed.startOnLatest);
      if (typeof parsed.openMatrixByDefault === "boolean") setOpenMatrixByDefault(parsed.openMatrixByDefault);
    } catch {
      window.localStorage.removeItem("v6-studio-ui-settings");
    }
  }, []);

  useEffect(() => {
    window.localStorage.setItem(
      "v6-studio-ui-settings",
      JSON.stringify({
        defaultLanding,
        density,
        fontScale,
        glass,
        language,
        motion,
        openMatrixByDefault,
        startOnLatest,
        warningsFirst
      })
    );
  }, [defaultLanding, density, fontScale, glass, language, motion, openMatrixByDefault, startOnLatest, warningsFirst]);

  function openLatestRun(): void {
    const latest = runs[0];
    if (latest) {
      setSelectedRunKey(runKey(latest));
    }
    setMode("runs");
    setDetailTab("charts");
  }

  function enterStudio(): void {
    setMode("runs");
    setDetailTab("charts");
  }

  useEffect(() => {
    void i18n.changeLanguage(language);
  }, [language]);

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
          if (openMatrixByDefault) {
            setMode("matrix");
          } else if (startOnLatest && loadedRuns[0]) {
            setMode("runs");
            setDetailTab("charts");
          } else if (!defaultLanding) {
            setMode("runs");
          }
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
    setDetailTab("charts");

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
    <AppShell density={density} fontScale={fontScale} glass={glass} motion={motion}>
      <Sidebar
        error={runsError}
        mode={mode}
        runs={filteredRuns}
        runsStatus={runsStatus}
        search={runSearch}
        selectedRunKey={selectedRunKey}
        onChangeMode={setMode}
        onOpenSettings={() => setSettingsOpen(true)}
        onSearch={setRunSearch}
        onSelectRun={(run) => setSelectedRunKey(runKey(run))}
      />

      <section
        className={mode === "landing" ? "workspace workspace--landing" : "workspace"}
        aria-label={mode === "runs" ? "Report Workspace" : mode === "matrix" ? "Regression Matrix Workspace" : "Studio Landing"}
      >
        <StudioTopBar
          density={density}
          language={language}
          mode={mode}
          motion={motion}
          quickSearch={runSearch}
          onOpenSettings={() => setSettingsOpen(true)}
          onQuickSearch={(value) => {
            setRunSearch(value);
            if (mode === "landing") {
              setMode("runs");
            }
          }}
          onSetDensity={setDensity}
          onSetLanguage={setLanguage}
        />

        {mode === "landing" ? (
          <LandingHero onEnter={enterStudio} onOpenLatest={openLatestRun} onOpenMatrix={() => setMode("matrix")} />
        ) : mode === "runs" ? (
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
            <span>{t("noExternalApi")}</span>
          </aside>
        ) : null}
      </section>

      <InsightRail detail={activeRunDetail} warningsFirst={warningsFirst} />

      <aside className="studio-system-card" aria-label="App information">
        <AppInfoCard appInfo={appInfo} error={appInfoError} status={appInfoStatus} />
      </aside>

      <SettingsCenter
        defaultLanding={defaultLanding}
        density={density}
        fontScale={fontScale}
        glass={glass}
        language={language}
        motion={motion}
        open={settingsOpen}
        warningsFirst={warningsFirst}
        onClose={() => setSettingsOpen(false)}
        onDefaultLanding={(value) => {
          setDefaultLanding(value);
          if (value) {
            setMode("landing");
          }
        }}
        onDensity={setDensity}
        onFontScale={setFontScale}
        onGlass={setGlass}
        onLanguage={setLanguage}
        onMatrixDefault={setOpenMatrixByDefault}
        onMotion={setMotion}
        onStartLatest={setStartOnLatest}
        onWarningsFirst={setWarningsFirst}
        openMatrixByDefault={openMatrixByDefault}
        startOnLatest={startOnLatest}
      />
    </AppShell>
  );
}
