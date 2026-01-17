// hooks/useExamAnalysis.ts
import { useState, useCallback } from "react";
import { analyzeDocx } from "../services/tauri/analyzeDocx";
import { useMixStore } from "../store/mixStore";
import { ERROR_CODES, ERROR_MESSAGES } from "../constants/exam";

/**
 * Custom hook for managing exam file analysis
 * Encapsulates analysis logic, loading state, and error handling
 */
export function useExamAnalysis() {
  const { selectedFilePath, jobId: cachedJobId, setJobId } = useMixStore();
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Analyze a DOCX file or use cached result
   */
  const analyze = useCallback(
    async (sourcePath: string) => {
      if (!sourcePath) {
        setError(ERROR_MESSAGES[ERROR_CODES.INVALID_FILE_PATH]);
        return { success: false, error: ERROR_MESSAGES[ERROR_CODES.INVALID_FILE_PATH] };
      }

      // Check if already analyzed
      if (cachedJobId && selectedFilePath === sourcePath) {
        console.log("Using cached analysis, jobId:", cachedJobId);
        return { success: true, jobId: cachedJobId };
      }

      setIsAnalyzing(true);
      setError(null);

      try {
        const jobId = crypto.randomUUID();
        const result = await analyzeDocx({ jobId, sourcePath });

        console.log("analyze_docx result", result);

        if (result.ok) {
          setJobId(result.jobId);
          setIsAnalyzing(false);
          return { success: true, jobId: result.jobId };
        }

        const errorMsg = ERROR_MESSAGES[ERROR_CODES.ANALYSIS_FAILED];
        setIsAnalyzing(false);
        setError(errorMsg);
        return { success: false, error: errorMsg };
      } catch (err) {
        const message =
          err instanceof Error ? err.message : String(err ?? "Unknown error");
        setIsAnalyzing(false);
        setError(message);
        return { success: false, error: message };
      }
    },
    [cachedJobId, selectedFilePath, setJobId]
  );

  /**
   * Reset error state
   */
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    analyze,
    isAnalyzing,
    error,
    clearError,
  };
}
