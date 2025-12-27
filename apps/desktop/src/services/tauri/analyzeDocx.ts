import { invoke } from "@tauri-apps/api/core";

export type AnalyzeDocxPayload = {
  jobId: string;
  sourcePath: string;
};

export type AnalyzeDocxResult = {
  ok: boolean;
  jobId: string;
  workspacePath: string;
};

export async function analyzeDocx(
  payload: AnalyzeDocxPayload,
): Promise<AnalyzeDocxResult> {
  return invoke<AnalyzeDocxResult>("analyze_docx", payload);
}
