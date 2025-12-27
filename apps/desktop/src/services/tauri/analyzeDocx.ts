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
  // Tauri command nhận tham số tên là `payload`, nên cần wrap lại.
  return invoke<AnalyzeDocxResult>("analyze_docx", { payload });
}
