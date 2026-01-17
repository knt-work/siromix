import type { FC } from "react";
import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { FlowNavigation } from "../../components/FlowNavigation";
import { ProgressModal, type ProgressStage } from "../../components/ProgressModal";
import { renderSegments } from "../../components/shared/SegmentRenderer";
import { AcademicCapIcon, XMarkIcon, CheckIcon } from "@heroicons/react/24/outline";
import { useMixStore, type ParsedDoc } from "../../store/mixStore";
import { mixExams } from "../../services/tauri/mixExams";
import { MIX_PROGRESS_STAGES } from "../../constants/exam";

export const PreviewPage: FC = () => {
  const { jobId } = useParams<{ jobId: string }>();
  const navigate = useNavigate();
  
  // Use Zustand store
  const { parsedData: cachedParsedData, setParsedData, setMixedExams, numVariants, examMetadata } = useMixStore();

  const [parsed, setParsed] = useState<ParsedDoc | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [showConfirmModal, setShowConfirmModal] = useState<boolean>(false);
  const [showProgress, setShowProgress] = useState<boolean>(false);
  const [progressStages, setProgressStages] = useState<ProgressStage[]>([]);

  useEffect(() => {
    if (!jobId) return;

    let isCancelled = false;
    
    // Check if we have cached data first
    if (cachedParsedData) {
      console.log("Using cached parsed data");
      setParsed(cachedParsedData);
      setLoading(false);
      return;
    }
    
    // No cache, fetch from backend
    setLoading(true);
    setError(null);

    invoke<ParsedDoc>("get_parsed", { jobId })
      .then((doc) => {
        if (isCancelled) return;
        console.log("Parsed doc:", doc);
        console.log("Number of questions:", doc.questions?.length);
        setParsed(doc);
        setParsedData(doc);
        setLoading(false);
      })
      .catch((err) => {
        if (isCancelled) return;
        const message =
          err instanceof Error ? err.message : String(err ?? "Unknown error");
        setError(message);
        setLoading(false);
      });

    return () => {
      isCancelled = true;
    };
  }, [jobId, cachedParsedData, setParsedData]);

  const handleConfirmMix = async () => {
    if (!parsed) return;

    setShowConfirmModal(false);
    setShowProgress(true);

    try {
      // Animate through progress stages
      for (let i = 0; i < MIX_PROGRESS_STAGES.length; i++) {
        const newStages = MIX_PROGRESS_STAGES.map((stage, idx) => ({
          label: stage.label,
          done: idx < i,
          current: idx === i,
        }));
        setProgressStages(newStages);
        await new Promise((resolve) => setTimeout(resolve, MIX_PROGRESS_STAGES[i].duration));
      }

      // Call Rust backend to mix exams (much faster than JS)
      const variants = await mixExams(parsed, numVariants);

      // Mark final stage as complete
      setProgressStages(
        MIX_PROGRESS_STAGES.map((stage, idx) => ({
          label: stage.label,
          done: true,
          current: idx === MIX_PROGRESS_STAGES.length - 1,
        }))
      );

      // Save to store
      setMixedExams(variants);

      await new Promise((resolve) => setTimeout(resolve, 400));

      // Navigate to result
      setShowProgress(false);
      navigate("/result");
    } catch (error) {
      console.error("Mix error:", error);
      setShowProgress(false);
      // TODO: Show error modal with user-friendly message
    }
  };

  return (
    <div className="min-h-screen">
      {/* Progress Modal */}
      <ProgressModal
        isOpen={showProgress}
        stages={progressStages}
        message={`Đang tạo ${numVariants} đề thi...`}
      />

      {/* Confirm Modal */}
      {showConfirmModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 backdrop-blur-sm">
          <div className="relative mx-4 w-full max-w-lg rounded-3xl border border-slate-200/60 bg-white shadow-2xl shadow-slate-900/20">
            <button
              onClick={() => setShowConfirmModal(false)}
              className="absolute right-4 top-4 rounded-full p-1.5 text-slate-400 transition hover:bg-slate-100 hover:text-slate-600"
            >
              <XMarkIcon className="h-5 w-5" />
            </button>

            <div className="px-8 py-8">
              <div className="mb-5 flex justify-center">
                <div className="inline-flex h-14 w-14 items-center justify-center rounded-2xl bg-violet-100">
                  <AcademicCapIcon className="h-8 w-8 text-violet-600" />
                </div>
              </div>

              <h2 className="text-center text-2xl font-bold text-slate-900">
                Xác nhận trộn đề
              </h2>

              <p className="mt-3 text-center text-sm text-slate-600">
                Bạn có chắc chắn muốn thực hiện trộn đề ngay?
              </p>

              <div className="mt-6 space-y-3 rounded-2xl bg-slate-50 p-5">
                <div className="flex justify-between text-sm">
                  <span className="font-medium text-slate-700">Tên kì thi:</span>
                  <span className="font-semibold text-slate-900">{examMetadata?.examName || "—"}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="font-medium text-slate-700">Môn thi:</span>
                  <span className="font-semibold text-slate-900">{examMetadata?.subject || "—"}</span>
                </div>
                {examMetadata?.gradeLevel && (
                  <div className="flex justify-between text-sm">
                    <span className="font-medium text-slate-700">Khối:</span>
                    <span className="font-semibold text-slate-900">{examMetadata.gradeLevel}</span>
                  </div>
                )}
                <div className="flex justify-between text-sm">
                  <span className="font-medium text-slate-700">Thời gian thi:</span>
                  <span className="font-semibold text-slate-900">{examMetadata?.durationMinutes || "—"} phút</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="font-medium text-slate-700">Tên trường:</span>
                  <span className="font-semibold text-slate-900">{examMetadata?.schoolName || "—"}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="font-medium text-slate-700">Số đề cần trộn:</span>
                  <span className="font-semibold text-slate-900">{numVariants} đề</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="font-medium text-slate-700">Mã đề:</span>
                  <span className="font-semibold text-slate-900">
                    {examMetadata?.customExamCodes?.join(", ") || "—"}
                  </span>
                </div>
              </div>

              <div className="mt-8 flex items-center gap-3">
                <button
                  onClick={() => setShowConfirmModal(false)}
                  className="inline-flex flex-1 items-center justify-center gap-2 rounded-full border border-slate-300 bg-white px-6 py-3.5 text-sm font-bold text-slate-700 shadow-sm transition hover:bg-slate-50 focus:outline-none focus:ring-4 focus:ring-slate-200"
                >
                  <XMarkIcon className="h-5 w-5" />
                  <span>Chờ chút</span>
                </button>
                <button
                  onClick={handleConfirmMix}
                  className="inline-flex flex-1 items-center justify-center gap-2 rounded-full bg-violet-600 px-6 py-3.5 text-sm font-bold text-white shadow-lg shadow-violet-200 transition hover:bg-violet-700 focus:outline-none focus:ring-4 focus:ring-violet-200"
                >
                  <CheckIcon className="h-5 w-5" />
                  <span>Đúng</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Background */}
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
                    Xem trước đề đã nhập
                  </h1>
                  <p className="mt-3 text-base text-slate-600">
                    Job ID: <span className="font-mono text-slate-800">{jobId}</span>
                  </p>

                  {loading && (
                    <p className="mt-6 text-sm text-slate-600">Đang tải dữ liệu đề...</p>
                  )}

                  {error && !loading && (
                    <p className="mt-6 text-sm text-rose-600">
                      Không tải được dữ liệu đề: {error}
                    </p>
                  )}

                  {!loading && !error && parsed && (
                    <>
                      {/* Question List - Temporarily disabled virtualization for debugging */}
                      <div className="mt-8 space-y-6">
                        {parsed.questions && parsed.questions.length > 0 ? (
                          parsed.questions.map((q) => (
                            <div
                              key={q.number}
                              className="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-200"
                            >
                              <div className="flex items-start gap-2">
                                <div className="mt-1 text-sm font-semibold text-slate-700">
                                  Câu {q.number}.
                                </div>
                                <div className="flex-1 text-sm text-slate-800">
                                  {renderSegments(q.stem)}
                                </div>
                              </div>

                              <div className="mt-3 space-y-1.5 text-sm">
                                {q.options.map((opt) => {
                                  const isCorrect =
                                    q.correct_label && opt.label === q.correct_label;
                                  return (
                                    <div
                                      key={opt.label}
                                      className={`flex items-start gap-2 rounded-lg px-2 py-1 ${
                                        isCorrect
                                          ? "bg-emerald-50 font-semibold text-emerald-800"
                                          : ""
                                      }`}
                                    >
                                      <div className="mt-0.5 w-6 shrink-0 text-slate-700">
                                        {opt.label}.
                                      </div>
                                      <div className="flex-1 text-slate-800">
                                        {opt.content.length === 0 ? (
                                          <span className="italic text-slate-400">
                                            (Trống)
                                          </span>
                                        ) : (
                                          renderSegments(opt.content)
                                        )}
                                      </div>
                                    </div>
                                  );
                                })}
                              </div>
                            </div>
                          ))
                        ) : (
                          <div className="rounded-2xl bg-white p-5 shadow-sm ring-1 ring-slate-200">
                            <p className="text-sm text-slate-600">
                              Không tìm thấy câu hỏi nào trong file.
                            </p>
                          </div>
                        )}
                      </div>

                      {/* Spacer for fixed navigation */}
                      <div className="h-10"></div>
                    </>
                  )}
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
                    onBack={() => navigate("/")}
                    onNext={() => {
                      setShowConfirmModal(true);
                    }}
                    backLabel="Quay lại"
                    nextLabel="Trộn ngay"
                    subtitle="Xử lý 100% offline • Export .docx + .xlsx"
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
