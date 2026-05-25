import { invoke } from "@tauri-apps/api/core";
import type { AppInfo, RunDetail, RunSummary } from "../types/app";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function listRuns(): Promise<RunSummary[]> {
  return invoke<RunSummary[]>("list_runs");
}

export async function loadRunDetail(ticker: string, runId: string): Promise<RunDetail> {
  return invoke<RunDetail>("load_run_detail", { ticker, runId });
}
