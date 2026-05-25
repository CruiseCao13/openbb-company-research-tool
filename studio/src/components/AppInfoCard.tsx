import type { AppInfo, AppInfoStatus } from "../types/app";

type AppInfoCardProps = {
  appInfo: AppInfo | null;
  error: string | null;
  status: AppInfoStatus;
};

function statusLabel(status: AppInfoStatus): string {
  if (status === "connected") {
    return "IPC connected";
  }
  if (status === "failed") {
    return "IPC failed";
  }
  if (status === "browser-preview") {
    return "Fallback dev warning";
  }
  return "Loading app info...";
}

export function AppInfoCard({ appInfo, error, status }: AppInfoCardProps): JSX.Element {
  return (
    <article className="app-info-card">
      <div className="card-header">
        <span className="card-label">App Info</span>
        <span className={`ipc-state ipc-state--${status}`}>{statusLabel(status)}</span>
      </div>

      {appInfo ? (
        <dl className="app-info-list">
          <div>
            <dt>App version</dt>
            <dd>{appInfo.app_version}</dd>
          </div>
          <div>
            <dt>Repo root</dt>
            <dd>Available</dd>
          </div>
          <div>
            <dt>Reports root</dt>
            <dd>Available</dd>
          </div>
          <div>
            <dt>Platform</dt>
            <dd>{appInfo.platform}</dd>
          </div>
          <div>
            <dt>Studio mode</dt>
            <dd>{appInfo.studio_mode}</dd>
          </div>
          <div>
            <dt>IPC status</dt>
            <dd>Connected through Tauri command</dd>
          </div>
          <details className="advanced-path-details">
            <summary>Advanced paths</summary>
            <code>{appInfo.repo_root}</code>
            <code>{appInfo.reports_root}</code>
          </details>
        </dl>
      ) : (
        <div className="app-info-empty">
          <strong>{statusLabel(status)}</strong>
          <span>{error ?? "Waiting for the Tauri backend to return app and repo metadata."}</span>
        </div>
      )}
    </article>
  );
}
