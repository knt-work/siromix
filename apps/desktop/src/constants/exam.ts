// constants/exam.ts
/**
 * Default number of exam variants to generate
 */
export const DEFAULT_NUM_VARIANTS = 4;

/**
 * Default exam duration in minutes
 */
export const DEFAULT_DURATION = 90;

/**
 * Available option labels for multiple choice questions
 */
export const OPTION_LABELS = ["A", "B", "C", "D", "E", "F"] as const;

/**
 * Minimum value for generated exam codes
 */
export const EXAM_CODE_MIN = 100;

/**
 * Maximum value for generated exam codes
 */
export const EXAM_CODE_MAX = 999;

/**
 * Regular expression patterns for parsing document
 */
export const PATTERNS = {
  QUESTION: /^(Câu|Question)\s+(\d+)\./,
  OPTION: /^(?<label>#?[A-F])\s*\./,
} as const;

/**
 * Validation error codes
 */
export const ERROR_CODES = {
  NO_FILE_SELECTED: "NO_FILE_SELECTED",
  INVALID_FILE_PATH: "INVALID_FILE_PATH",
  ANALYSIS_FAILED: "ANALYSIS_FAILED",
  NO_PARSED_DATA: "NO_PARSED_DATA",
  EXPORT_FAILED: "EXPORT_FAILED",
} as const;

/**
 * User-friendly error messages
 */
export const ERROR_MESSAGES = {
  [ERROR_CODES.NO_FILE_SELECTED]:
    "Vui lòng chọn file .docx trước khi tiếp tục.",
  [ERROR_CODES.INVALID_FILE_PATH]:
    "Không lấy được đường dẫn file nguồn từ input.",
  [ERROR_CODES.ANALYSIS_FAILED]: "Phân tích đề không thành công.",
  [ERROR_CODES.NO_PARSED_DATA]: "Không tìm thấy dữ liệu đề đã phân tích.",
  [ERROR_CODES.EXPORT_FAILED]: "Lỗi khi export file.",
} as const;

/**
 * LocalStorage keys
 */
export const STORAGE_KEYS = {
  MIX_STORE: "siromix-mix-storage",
} as const;

/**
 * Progress stages for mixing operation
 */
export const MIX_PROGRESS_STAGES = [
  { label: "Đảo câu hỏi", duration: 600 },
  { label: "Đảo đáp án", duration: 600 },
  { label: "Tạo mã đề", duration: 600 },
  { label: "Hoàn tất", duration: 400 },
] as const;

/**
 * File extensions
 */
export const FILE_EXTENSIONS = {
  DOCX: ".docx",
  XLSX: ".xlsx",
} as const;

/**
 * Default exam metadata
 */
export const DEFAULT_EXAM_METADATA = {
  examName: "",
  subject: "",
  durationMinutes: DEFAULT_DURATION,
} as const;

/**
 * Default exam codes for 4 variants
 */
export const DEFAULT_EXAM_CODES = ["132", "209", "357", "485"] as const;
