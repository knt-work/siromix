import type { FC } from "react";
import { useNavigate } from "react-router-dom";
import { AcademicCapIcon, DocumentArrowDownIcon } from "@heroicons/react/24/outline";
import { FlowNavigation } from "../../components/FlowNavigation";
import { AnswerKeyTable } from "./components/AnswerKeyTable";
import { useMixStore } from "../../store/mixStore";

export const MixedResultPage: FC = () => {
  const navigate = useNavigate();
  const { mixedExams, parsedData, jobId } = useMixStore();

  // If no mixed exams, redirect back
  if (!mixedExams || mixedExams.length === 0) {
    navigate(`/preview/${jobId || ""}`);
    return null;
  }

  // Get original answers from parsed data
  const originalAnswers =
    parsedData?.questions.map((q) => q.correct_label) || [];

  const numQuestions = mixedExams[0]?.questions.length || 0;
  const numVariants = mixedExams.length;

  return (
    <div className="min-h-screen">
      {/* Background giống design: xanh + sọc chéo */}
      <div className="min-h-screen bg-[radial-gradient(1200px_600px_at_20%_0%,rgba(255,255,255,0.9),rgba(255,255,255,0.35),rgba(255,255,255,0)_70%),linear-gradient(135deg,#8fd3ff_0%,#36b9ff_35%,#1aa7ff_55%,#1296f0_100%)]">
        <div className="min-h-screen bg-[repeating-linear-gradient(135deg,rgba(255,255,255,0.10)_0,rgba(255,255,255,0.10)_24px,rgba(255,255,255,0.03)_24px,rgba(255,255,255,0.03)_52px)]">
          {/* Top bar */}
          <header className="fixed left-0 right-0 top-0 z-50 flex items-center justify-between border-b border-white/30 bg-white/40 px-6 py-4 shadow-lg shadow-slate-900/5 backdrop-blur-md">
            <div className="flex items-center gap-2">
              <span className="inline-flex h-9 w-9 items-center justify-center rounded-xl bg-violet-600 shadow-md shadow-violet-200">
                <AcademicCapIcon className="h-5 w-5 text-white" />
              </span>
              <div className="text-lg font-bold text-violet-700">SiroMix</div>
            </div>

            <div className="text-sm font-medium text-white/80 drop-shadow">
              Trộn đề offline-first
            </div>
          </header>

          {/* Main */}
          <main className="mx-auto flex min-h-[calc(100vh-104px)] max-w-6xl items-start justify-center px-6 pb-32 pt-24">
            <div className="w-full max-w-5xl">
              {/* Card */}
              <div className="rounded-[28px] border border-slate-200/60 bg-white/90 shadow-2xl shadow-slate-900/10 backdrop-blur">
                <div className="px-10 py-10">
                  <h1 className="text-4xl font-extrabold tracking-tight text-slate-900">
                    Kết quả trộn đề
                  </h1>

                  {/* Stats Summary */}
                  <div className="mt-4 flex flex-wrap items-center gap-4 text-sm text-slate-600">
                    <div className="flex items-center gap-2">
                      <span className="text-lg">📋</span>
                      <span className="font-semibold">
                        {numVariants} đề thi
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-lg">🔢</span>
                      <span className="font-semibold">
                        {numQuestions} câu hỏi
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-lg">⏱️</span>
                      <span className="font-semibold">90 phút</span>
                    </div>
                  </div>

                  {/* Exam Codes */}
                  <div className="mt-6 rounded-2xl bg-violet-50 p-5">
                    <div className="text-sm font-semibold text-violet-700">
                      Mã đề đã tạo:
                    </div>
                    <div className="mt-2 flex flex-wrap gap-2">
                      {mixedExams.map((exam) => (
                        <span
                          key={exam.examCode}
                          className="inline-flex items-center rounded-full bg-violet-600 px-4 py-1.5 font-mono text-sm font-bold text-white shadow-md shadow-violet-200"
                        >
                          {exam.examCode}
                        </span>
                      ))}
                    </div>
                  </div>

                  {/* Answer Key Table */}
                  <div className="mt-8">
                    <h2 className="mb-4 text-xl font-bold text-slate-900">
                      Bảng đáp án
                    </h2>
                    <AnswerKeyTable
                      mixedExams={mixedExams}
                      originalAnswers={originalAnswers}
                    />
                  </div>

                  {/* Download Info */}
                  <div className="mt-8 rounded-2xl border border-emerald-200 bg-emerald-50 p-5">
                    <div className="flex items-start gap-3">
                      <DocumentArrowDownIcon className="h-6 w-6 shrink-0 text-emerald-600" />
                      <div>
                        <div className="text-sm font-semibold text-emerald-800">
                          Sẵn sàng export
                        </div>
                        <div className="mt-1 text-xs text-emerald-700">
                          {numVariants} file DOCX (đề thi) + 1 file XLSX (đáp
                          án) • Xử lý 100% offline
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Spacer for fixed navigation */}
                  <div className="h-10"></div>
                </div>
              </div>
            </div>
          </main>

          {/* Fixed Navigation */}
          <div className="fixed bottom-0 left-0 right-0 z-50 border-t border-slate-200/60 bg-white/90 shadow-2xl shadow-slate-900/20 backdrop-blur-md">
            <div className="mx-auto max-w-6xl px-6">
              <div className="max-w-5xl">
                <div className="px-10 py-6">
                  <FlowNavigation
                    onBack={() => navigate(`/preview/${jobId || ""}`)}
                    onNext={() => {
                      // TODO: implement export logic
                      console.log("Export clicked");
                    }}
                    backLabel="Xem lại"
                    nextLabel="Tải xuống"
                    subtitle="Export DOCX + XLSX • Lưu vào máy"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
