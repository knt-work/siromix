// pages/MixedResult/components/AnswerKeyTable.tsx
import type { FC } from "react";
import type { MixedExam } from "../../../lib/mixAlgorithm";

export interface AnswerKeyTableProps {
  mixedExams: MixedExam[];
  originalAnswers: string[]; // Original answers from parsed doc
}

export const AnswerKeyTable: FC<AnswerKeyTableProps> = ({
  mixedExams,
  originalAnswers,
}) => {
  const numQuestions = mixedExams[0]?.questions.length || 0;

  return (
    <div className="overflow-x-auto rounded-2xl border border-slate-200">
      <table className="w-full border-collapse">
        <thead>
          <tr className="bg-slate-100">
            <th className="sticky left-0 z-10 border-b border-r border-slate-200 bg-slate-100 px-4 py-3 text-center text-sm font-bold text-slate-700">
              Câu hỏi
            </th>
            <th className="border-b border-r border-slate-200 bg-slate-50 px-4 py-3 text-center text-sm font-bold text-slate-700">
              Đề gốc
            </th>
            {mixedExams.map((exam) => (
              <th
                key={exam.examCode}
                className="border-b border-slate-200 px-4 py-3 text-center text-sm font-bold text-slate-700"
              >
                Đề {exam.examCode}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: numQuestions }).map((_, idx) => {
            const questionNumber = idx + 1;

            return (
              <tr
                key={questionNumber}
                className="transition hover:bg-slate-50/50"
              >
                {/* Question Number */}
                <td className="sticky left-0 z-10 border-b border-r border-slate-200 bg-white px-4 py-3 text-center font-semibold text-slate-900">
                  {questionNumber}
                </td>

                {/* Original Answer */}
                <td className="border-b border-r border-slate-200 bg-slate-50/50 px-4 py-3 text-center font-semibold text-slate-700">
                  {originalAnswers[idx] || "-"}
                </td>

                {/* Mixed Exam Answers */}
                {mixedExams.map((exam) => {
                  // Find the question that has displayNumber === questionNumber
                  const question = exam.questions.find(
                    (q) => q.displayNumber === questionNumber
                  );

                  return (
                    <td
                      key={exam.examCode}
                      className="border-b border-slate-200 px-4 py-3 text-center font-semibold text-slate-900"
                    >
                      {question?.correctAnswer || "-"}
                    </td>
                  );
                })}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};
