import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  ArtifactActionResult,
  QualityMatrix,
  RunDetail,
  RunSummary,
  TrainingRunSummary
} from "../types/app";

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (
    typeof invoke !== "function" ||
    typeof window === "undefined" ||
    !("__TAURI_INTERNALS__" in window)
  ) {
    throw new Error("__TAURI__ invoke unavailable in browser preview");
  }
  return invoke<T>(command, args);
}

export async function getAppInfo(): Promise<AppInfo> {
  return invokeCommand<AppInfo>("get_app_info");
}

export async function listRuns(): Promise<RunSummary[]> {
  return invokeCommand<RunSummary[]>("list_runs");
}

export async function loadRunDetail(ticker: string, runId: string): Promise<RunDetail> {
  return invokeCommand<RunDetail>("load_run_detail", { ticker, runId });
}

export async function openArtifact(path: string): Promise<ArtifactActionResult> {
  return invokeCommand<ArtifactActionResult>("open_artifact", { path });
}

export async function revealInFolder(path: string): Promise<ArtifactActionResult> {
  return invokeCommand<ArtifactActionResult>("reveal_in_folder", { path });
}

export function artifactImageSrc(path: string): string {
  if (typeof convertFileSrc !== "function") {
    return "";
  }
  return convertFileSrc(path);
}

export async function listTrainingRuns(): Promise<TrainingRunSummary[]> {
  return invokeCommand<TrainingRunSummary[]>("list_training_runs");
}

export async function loadQualityMatrix(runId: string): Promise<QualityMatrix> {
  return invokeCommand<QualityMatrix>("load_quality_matrix", { runId });
}
