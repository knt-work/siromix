// lib/mixAlgorithm.ts
import type { ParsedDoc, Question, OptionItem } from "../store/mixStore";

/**
 * Mixed exam variant with unique code
 */
export type MixedExam = {
  examCode: string;
  questions: MixedQuestion[];
};

/**
 * Question in a mixed exam
 */
export type MixedQuestion = {
  originalNumber: number;
  displayNumber: number;
  stem: any[];
  options: MixedOption[];
  correctAnswer: string;
};

/**
 * Option after shuffling
 */
export type MixedOption = {
  label: string;
  originalLabel: string;
  content: any[];
};

/**
 * Generate a random 3-digit exam code (100-999)
 */
export function generateExamCode(): string {
  return Math.floor(100 + Math.random() * 900).toString();
}

/**
 * Generate unique exam codes
 */
export function generateExamCodes(count: number): string[] {
  const codes = new Set<string>();
  while (codes.size < count) {
    codes.add(generateExamCode());
  }
  return Array.from(codes);
}

/**
 * Seeded random number generator for reproducible shuffles
 * Uses Linear Congruential Generator algorithm
 */
export function seededRandom(seed: number): () => number {
  let state = seed;
  return function () {
    // LCG parameters (common values)
    state = (state * 1664525 + 1013904223) % 2 ** 32;
    return state / 2 ** 32;
  };
}

/**
 * Fisher-Yates shuffle with seeded random
 */
export function shuffleArray<T>(array: T[], seed: number): T[] {
  const rng = seededRandom(seed);
  const shuffled = [...array];

  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(rng() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }

  return shuffled;
}

/**
 * Shuffle options within a question and return mapping
 */
function shuffleOptions(
  options: OptionItem[],
  seed: number
): { shuffled: MixedOption[]; mapping: Map<string, string> } {
  const shuffled = shuffleArray(options, seed);
  const labels = ["A", "B", "C", "D", "E", "F"];
  const mapping = new Map<string, string>();

  const result = shuffled.map((opt, idx) => {
    const newLabel = labels[idx];
    mapping.set(opt.label, newLabel);

    return {
      label: newLabel,
      originalLabel: opt.label,
      content: opt.content, // Keep segments intact
    };
  });

  return { shuffled: result, mapping };
}

/**
 * Main mix function - creates multiple exam variants
 * 
 * @param parsedDoc - Parsed document with questions
 * @param numVariants - Number of exam variants to generate (default: 4)
 * @returns Array of mixed exam variants with unique codes
 */
export function mixExam(
  parsedDoc: ParsedDoc,
  numVariants: number = 4
): MixedExam[] {
  const variants: MixedExam[] = [];
  const examCodes = generateExamCodes(numVariants);

  for (let v = 0; v < numVariants; v++) {
    const seed = Date.now() + v * 1000;

    // 1. Shuffle question order
    const shuffledQuestions = shuffleArray(parsedDoc.questions, seed);

    // 2. Process each question
    const mixedQuestions: MixedQuestion[] = [];

    shuffledQuestions.forEach((q, idx) => {
      // Shuffle options with different seed for each question
      const { shuffled, mapping } = shuffleOptions(q.options, seed + idx);

      // Find new correct answer label
      const newCorrectLabel = mapping.get(q.correct_label) || q.correct_label;

      mixedQuestions.push({
        originalNumber: q.number,
        displayNumber: idx + 1,
        stem: q.stem, // Keep segments intact
        options: shuffled,
        correctAnswer: newCorrectLabel,
      });
    });

    variants.push({
      examCode: examCodes[v],
      questions: mixedQuestions,
    });
  }

  return variants;
}
