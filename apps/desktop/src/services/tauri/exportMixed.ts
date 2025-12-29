// services/tauri/exportMixed.ts
import { invoke } from "@tauri-apps/api/core";
import type { MixedExam } from "../../lib/mixAlgorithm";

export interface ExportMixedParams {
  jobId: string;
  exams: MixedExam[];
  originalAnswers: string[];
  outputDir: string;
}

export interface ExportResponse {
  success: boolean;
  docxFiles: string[];
  xlsxFile: string;
  outputDirectory: string;
}

export async function exportMixedExams(
  params: ExportMixedParams
): Promise<ExportResponse> {
  return invoke<ExportResponse>("export_mixed_exams", {
    jobId: params.jobId,
    exams: params.exams,
    originalAnswers: params.originalAnswers,
    outputDir: params.outputDir,
  });
}
