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

export type RunDetail = {
  ticker: string;
  run_id: string;
  run_folder: string;
  status: {
    overall_status: string | null;
    provider_status: string | null;
    visual_lint_status: string | null;
    pdf_export_status: string | null;
    human_review_required: boolean | null;
  };
  ai_usage: {
    source: string | null;
    external_ai_used: boolean | null;
    local_mock_used: boolean | null;
    cache_hits: number | null;
    new_external_ai_calls: number | null;
    model: string | null;
    prompt_versions: string[];
  };
  provider: {
    provider: string | null;
    source: string | null;
    provider_adapter: string | null;
    package_used: boolean | null;
    mock: boolean | null;
    market: string | null;
    currency: string | null;
    limitations: string[];
    missing_fields: string[];
  };
  company: {
    name: string | null;
    identity: string | null;
    frame: string | null;
    not_this: string[];
    confidence: string | null;
  };
  financial_interpretation: {
    revenue_explanation: string | null;
    margin_explanation: string | null;
    cash_flow_explanation: string | null;
    where_money_comes_from: string | null;
    where_money_goes: string | null;
    debt_and_financing: string | null;
    valuation_method_fit: string | null;
  };
  blueprint: {
    core_thesis: string | null;
    asset_profile: string | null;
    must_analyze: string[];
    must_not_analyze_as_core: string[];
    key_questions: string[];
    red_flags: string[];
    data_gaps: string[];
    next_checks: string[];
  };
  self_review: {
    company_understanding_check: string | null;
    framework_fit_check: string | null;
    numeric_consistency_check: string | null;
    money_flow_check: string | null;
    final_confidence: string | null;
    human_review_required: boolean | null;
  };
  charts: Array<{
    title: string;
    image_path: string | null;
    image_exists: boolean;
    source: string | null;
    status: string | null;
    why_selected: string | null;
    what_to_look_at: string | null;
    what_it_means: string | null;
    what_not_to_overread: string | null;
    next_check: string | null;
  }>;
  artifacts: {
    markdown_report_path: string | null;
    pdf_report_path: string | null;
    dashboard_path: string | null;
    ai_usage_path: string | null;
    blueprint_path: string | null;
    validator_report_path: string | null;
    provider_payload_path: string | null;
  };
  audit_trail: Array<{
    stage: string;
    label: string;
    status: "pending" | "running" | "pass" | "warning" | "fail" | "skipped" | "cached" | "unknown";
    source: string | null;
    message: string | null;
    artifact_path: string | null;
  }>;
  warnings: string[];
};

export type RunDetailStatus = "idle" | "loading" | "ready" | "error" | "browser-preview";

export type ArtifactActionResult = {
  ok: boolean;
  action: "open" | "reveal";
  path: string;
  message: string;
};
