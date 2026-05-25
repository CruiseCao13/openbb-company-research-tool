import { invoke } from "@tauri-apps/api/core";
import type { AppInfo, RunSummary } from "../types/app";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function listRuns(): Promise<RunSummary[]> {
  return invoke<RunSummary[]>("list_runs");
}
