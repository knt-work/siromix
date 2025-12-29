// src/store/mixStore.ts
import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { MixedExam } from "../lib/mixAlgorithm";

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
  
  // Number of exam variants
  numVariants: number;
  
  // Actions
  setSelectedFile: (path: string | null) => void;
  setJobId: (id: string | null) => void;
  setParsedData: (data: ParsedDoc | null) => void;
  setIsAnalyzing: (analyzing: boolean) => void;
  setMixedExams: (exams: MixedExam[] | null) => void;
  setNumVariants: (num: number) => void;
  
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
      numVariants: 4,
      
      // Actions
      setSelectedFile: (path) => set({ selectedFilePath: path }),
      
      setJobId: (id) => set({ jobId: id }),
      
      setParsedData: (data) => set({ parsedData: data }),
      
      setIsAnalyzing: (analyzing) => set({ isAnalyzing: analyzing }),
      
      setMixedExams: (exams) => set({ mixedExams: exams }),
      
      setNumVariants: (num) => set({ numVariants: num }),
      
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
          parsedData: null,
          isAnalyzing: false,
          mixedExams: null,
          numVariants: 4,
        }),
    }),
    {
      name: "siromix-mix-storage", // localStorage key
      // Only persist these fields
      partialize: (state) => ({
        selectedFilePath: state.selectedFilePath,
        jobId: state.jobId,
        parsedData: state.parsedData,
        mixedExams: state.mixedExams,
        numVariants: state.numVariants,
      }),
    }
  )
);

// Export types for reuse
export type { ParsedDoc, Question, OptionItem, Segment };
