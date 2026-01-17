// services/tauri/mixExams.ts
import { invoke } from "@tauri-apps/api/core";
import type { ParsedDoc } from "../../store/mixStore";

/**
 * Mixed exam structure returned from Rust
 */
export interface MixedExam {
  examCode: string;
  questions: MixedQuestion[];
}

export interface MixedQuestion {
  originalNumber: number;
  displayNumber: number;
  stem: any[];
  options: MixedOption[];
  correctAnswer: string;
}

export interface MixedOption {
  label: string;
  originalLabel: string;
  content: any[];
}

/**
 * Call Rust backend to mix exams
 * This is much faster than the JavaScript implementation for large documents
 */
export async function mixExams(
  parsedDoc: ParsedDoc,
  numVariants: number
): Promise<MixedExam[]> {
  return invoke<MixedExam[]>("mix_exams", {
    parsedDoc,
    numVariants,
  });
}
