// src/store/mixStore.ts
import { create } from "zustand";
import { persist } from "zustand/middleware";
import { DEFAULT_NUM_VARIANTS, STORAGE_KEYS } from "../constants/exam";
import type { MixedExam } from "../services/tauri/mixExams";

// Types matching the ParsedDoc structure from PreviewPage
type Segment =
  | { type: "Text"; text: string }
  | { type: "Image"; asset_path: string }
  | { type: "Math"; omml: string };

type OptionItem = {
  label: string;
  locked: boolean;
  content: Segment[];
};

type Question = {
  number: number;
  stem: Segment[];
  options: OptionItem[];
  correct_label: string;
};

type ParsedDoc = {
  questions: Question[];
};

// Exam metadata
export interface ExamMetadata {
  examName: string;
  subject: string;
  gradeLevel?: string; // Khối (optional)
  durationMinutes: number;
  schoolName: string; // Tên trường (mandatory)
  numVariants: number;
  customExamCodes: string[];
}

interface MixState {
  // Current selected file path
  selectedFilePath: string | null;
  
  // Current job ID
  jobId: string | null;
  
  // Cached parsed data from the analysis
  parsedData: ParsedDoc | null;
  
  // Loading state
  isAnalyzing: boolean;
  
  // Mixed exams data
  mixedExams: MixedExam[] | null;
  
  // Exam metadata
  examMetadata: ExamMetadata | null;
  
  // Number of exam variants (deprecated - use examMetadata.numVariants)
  numVariants: number;
  
  // Actions
  setSelectedFile: (path: string | null) => void;
  setJobId: (id: string | null) => void;
  setParsedData: (data: ParsedDoc | null) => void;
  setIsAnalyzing: (analyzing: boolean) => void;
  setMixedExams: (exams: MixedExam[] | null) => void;
  setNumVariants: (num: number) => void;
  setExamMetadata: (metadata: ExamMetadata) => void;
  
  // Clear all data (e.g., when selecting a new file)
  clearAnalysis: () => void;
  
  // Reset everything
  reset: () => void;
}

export const useMixStore = create<MixState>()(
  persist(
    (set) => ({
      // Initial state
      selectedFilePath: null,
      jobId: null,
      parsedData: null,
      isAnalyzing: false,
      mixedExams: null,
      examMetadata: null,
      numVariants: DEFAULT_NUM_VARIANTS,
      
      // Actions
      setSelectedFile: (path) => set({ selectedFilePath: path }),
      
      setJobId: (id) => set({ jobId: id }),
      
      setParsedData: (data) => set({ parsedData: data }),
      
      setIsAnalyzing: (analyzing) => set({ isAnalyzing: analyzing }),
      
      setMixedExams: (exams) => set({ mixedExams: exams }),
      
      setNumVariants: (num) => set({ numVariants: num }),
      
      setExamMetadata: (metadata) => set({ examMetadata: metadata, numVariants: metadata.numVariants }),
      
      clearAnalysis: () =>
        set({
          jobId: null,
          parsedData: null,
          mixedExams: null,
        }),
      
      reset: () =>
        set({
          selectedFilePath: null,
          jobId: null,
          examMetadata: null,
          numVariants: DEFAULT_NUM_VARIANTS,
        }),
    }),
    {
      name: STORAGE_KEYS.MIX_STORE, // localStorage key
      // Only persist these fields
      partialize: (state) => ({
        selectedFilePath: state.selectedFilePath,
        jobId: state.jobId,
        parsedData: state.parsedData,
        mixedExams: state.mixedExams,
        examMetadata: state.examMetadata,
        numVariants: state.numVariants,
      }),
    }
  )
);

// Export types for reuse
export type { ParsedDoc, Question, OptionItem, Segment };
