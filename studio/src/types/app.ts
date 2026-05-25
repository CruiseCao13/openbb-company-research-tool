export type AppInfo = {
  app_version: string;
  repo_root: string;
  reports_root: string;
  platform: string;
  studio_mode: string;
};

export type AppInfoStatus = "loading" | "connected" | "failed" | "browser-preview";

export type RunSummary = {
  ticker: string;
  run_id: string;
  market: string | null;
  provider: string | null;
  status: string | null;
  ai_source: string | null;
  external_ai_used: boolean | null;
  local_mock_used: boolean | null;
  cache_hits: number | null;
  new_external_ai_calls: number | null;
  human_review_required: boolean | null;
  generated_at: string | null;
  report_path_exists: boolean;
  dashboard_path_exists: boolean;
  pdf_path_exists: boolean;
  run_folder: string;
};

export type RunsStatus = "loading" | "ready" | "empty" | "failed" | "browser-preview";
