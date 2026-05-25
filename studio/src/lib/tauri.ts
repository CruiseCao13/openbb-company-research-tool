import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  ArtifactActionResult,
  QualityMatrix,
  RunDetail,
  RunSummary,
  TrainingRunSummary
} from "../types/app";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function listRuns(): Promise<RunSummary[]> {
  return invoke<RunSummary[]>("list_runs");
}

export async function loadRunDetail(ticker: string, runId: string): Promise<RunDetail> {
  return invoke<RunDetail>("load_run_detail", { ticker, runId });
}

export async function openArtifact(path: string): Promise<ArtifactActionResult> {
  return invoke<ArtifactActionResult>("open_artifact", { path });
}

export async function revealInFolder(path: string): Promise<ArtifactActionResult> {
  return invoke<ArtifactActionResult>("reveal_in_folder", { path });
}

export function artifactImageSrc(path: string): string {
  return convertFileSrc(path);
}

export async function listTrainingRuns(): Promise<TrainingRunSummary[]> {
  return invoke<TrainingRunSummary[]>("list_training_runs");
}

export async function loadQualityMatrix(runId: string): Promise<QualityMatrix> {
  return invoke<QualityMatrix>("load_quality_matrix", { runId });
}
