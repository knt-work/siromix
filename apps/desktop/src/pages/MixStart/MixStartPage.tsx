// src/pages/MixStart/MixStartPage.tsx
import { useState, type ChangeEvent, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useNavigate } from "react-router-dom";
import { LoadingOverlay } from "../../components/LoadingOverlay";
import { analyzeDocx } from "../../services/tauri/analyzeDocx";
import {
  AcademicCapIcon,
  ClockIcon,
  DocumentDuplicateIcon,
  DocumentTextIcon,
  ArrowUpTrayIcon,
} from "@heroicons/react/24/outline";

export function MixStartPage() {
  const navigate = useNavigate();
  const [hasFile, setHasFile] = useState(false);
  const [isErrorModalOpen, setIsErrorModalOpen] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [sourcePath, setSourcePath] = useState<string | null>(null);

  const handleFileChange = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0] ?? null;
    setHasFile(!!file);
    // Trong Tauri, nên dùng dialog API để lấy đường dẫn tuyệt đối.
    // Trường hợp input file được dùng trong môi trường web thuần, ta chỉ có thể lấy `file.name`.
    setSourcePath(file ? file.name : null);
  };

  const handlePickFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "DOCX", extensions: ["docx"] }],
      });

      if (typeof selected === "string") {
        setSourcePath(selected);
        setHasFile(true);
        return;
      }

      // User bấm Hủy: không làm gì, coi như chưa chọn file
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error ?? "Unknown error");
      setErrorMessage(message);
      setIsErrorModalOpen(true);
    }
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (!hasFile) {
      setErrorMessage("Vui lòng chọn file .docx trước khi tiếp tục.");
      setIsErrorModalOpen(true);
      return;
    }

    if (!sourcePath) {
      setErrorMessage("Không lấy được đường dẫn file nguồn từ input.");
      setIsErrorModalOpen(true);
      return;
    }

    setIsAnalyzing(true);

    try {
      const jobId = crypto.randomUUID();
      const result = await analyzeDocx({ jobId, sourcePath });
      // Demo: tạm thời chỉ log ra console
      // eslint-disable-next-line no-console
      console.log("analyze_docx result", result);

      if (result.ok) {
        setIsAnalyzing(false);
        navigate(`/preview/${result.jobId}`);
        return;
      }

      setIsAnalyzing(false);
      setErrorMessage("Phân tích đề không thành công.");
      setIsErrorModalOpen(true);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error ?? "Unknown error");
      setIsAnalyzing(false);
      setErrorMessage(message);
      setIsErrorModalOpen(true);
    }
  };

  return (
    <div className="min-h-screen">
      {/* Background giống design: xanh + sọc chéo */}
      <div className="min-h-screen bg-[radial-gradient(1200px_600px_at_20%_0%,rgba(255,255,255,0.9),rgba(255,255,255,0.35),rgba(255,255,255,0)_70%),linear-gradient(135deg,#8fd3ff_0%,#36b9ff_35%,#1aa7ff_55%,#1296f0_100%)]">
        <div className="min-h-screen bg-[repeating-linear-gradient(135deg,rgba(255,255,255,0.10)_0,rgba(255,255,255,0.10)_24px,rgba(255,255,255,0.03)_24px,rgba(255,255,255,0.03)_52px)]">
          {/* Top bar */}
          <header className="flex items-center justify-between border-b border-white/30 bg-white/40 px-6 py-4 backdrop-blur-md">
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
          <main className="mx-auto flex min-h-[calc(100vh-104px)] max-w-6xl items-center justify-center px-6 py-10">
            <div className="w-full max-w-5xl">
              {/* Card */}
              <div className="rounded-[28px] border border-slate-200/60 bg-white/90 shadow-2xl shadow-slate-900/10 backdrop-blur">
                <div className="px-10 py-10">
                  <h1 className="text-4xl font-extrabold tracking-tight text-slate-900">
                    Trộn đề trong vài giây
                  </h1>
                  <p className="mt-3 text-base text-slate-600">
                    Nhập thông tin cơ bản, chọn file .docx theo template và bấm Tiếp theo.
                  </p>

                  <form className="mt-10 space-y-7" onSubmit={handleSubmit}>
                    {/* Row 1: Tên kì thi + Môn thi */}
                    <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
                      <div>
                        <label className="text-sm font-semibold text-slate-800">
                          Tên Kì thi<span className="text-rose-500">*</span>
                        </label>
                        <div className="relative mt-3">
                          <DocumentTextIcon className="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-400" />
                          <input
                            type="text"
                            placeholder="Ví dụ: HK1 2025—2026"
                            className="w-full rounded-full border border-slate-200 bg-white/80 py-3.5 pl-12 pr-5 text-slate-900 placeholder:text-slate-400 outline-none focus:border-violet-300 focus:ring-4 focus:ring-violet-100"
                          />
                        </div>
                      </div>

                      <div>
                        <label className="text-sm font-semibold text-slate-800">
                          Môn thi<span className="text-rose-500">*</span>
                        </label>
                        <div className="relative mt-3">
                          <AcademicCapIcon className="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-400" />
                          <input
                            type="text"
                            placeholder="Ví dụ: Toán / Anh / Vật lý..."
                            className="w-full rounded-full border border-slate-200 bg-white/80 py-3.5 pl-12 pr-5 text-slate-900 placeholder:text-slate-400 outline-none focus:border-violet-300 focus:ring-4 focus:ring-violet-100"
                          />
                        </div>
                      </div>
                    </div>

                    {/* Row 2: Số phút + Số lượng đề cần trộn */}
                    <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
                      <div>
                        <label className="text-sm font-semibold text-slate-800">
                          Số phút<span className="text-rose-500">*</span>
                        </label>
                        <div className="relative mt-3">
                          <ClockIcon className="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-400" />
                          <input
                            type="number"
                            min={1}
                            placeholder="Ví dụ: 45"
                            className="w-full rounded-full border border-slate-200 bg-white/80 py-3.5 pl-12 pr-5 text-slate-900 placeholder:text-slate-400 outline-none focus:border-violet-300 focus:ring-4 focus:ring-violet-100"
                          />
                        </div>
                      </div>

                      <div>
                        <label className="text-sm font-semibold text-slate-800">
                          Số lượng đề cần trộn<span className="text-rose-500">*</span>
                        </label>
                        <div className="relative mt-3">
                          <DocumentDuplicateIcon className="pointer-events-none absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-400" />
                          <input
                            type="number"
                            min={1}
                            placeholder="Ví dụ: 4"
                            className="w-full rounded-full border border-slate-200 bg-white/80 py-3.5 pl-12 pr-5 text-slate-900 placeholder:text-slate-400 outline-none focus:border-violet-300 focus:ring-4 focus:ring-violet-100"
                          />
                        </div>
                      </div>
                    </div>

                    {/* File upload */}
                    <div>
                      <label className="text-sm font-semibold text-slate-800">
                        File đề thô (.docx)
                      </label>

                      <div className="mt-3 rounded-2xl border border-dashed border-slate-300 bg-white/70 px-6 py-10">
                        <div className="flex flex-col items-center justify-center text-center">
                          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-violet-50">
                            <ArrowUpTrayIcon className="h-6 w-6 text-violet-600" />
                          </div>

                          <p className="mt-4 text-sm text-slate-600">
                            Kéo thả file .docx vào đây hoặc bấm để chọn file
                          </p>
                          <p className="mt-1 text-sm text-slate-700">
                            {sourcePath ? (
                              <span className="font-medium text-slate-800" title={sourcePath}>
                                {sourcePath.split(/[/\\]/).pop()}
                              </span>
                            ) : (
                              "Chưa có file nào được chọn."
                            )}
                          </p>

                          <label
                            className="mt-5 inline-flex cursor-pointer items-center rounded-full bg-violet-600 px-5 py-2.5 text-sm font-semibold text-white shadow-md shadow-violet-200 hover:bg-violet-700"
                            onClick={(event) => {
                              event.preventDefault();
                              void handlePickFile();
                            }}
                          >
                            Chọn file
                            <input
                              type="file"
                              accept=".docx"
                              className="hidden"
                              onChange={handleFileChange}
                            />
                          </label>
                        </div>
                      </div>

                      <p className="mt-3 text-xs text-slate-500">
                        Chỉ nhận .docx theo template SiroMix. Đáp án đúng là lựa chọn được gạch chân.
                      </p>
                    </div>

                    {/* CTA */}
                    <div className="pt-2">
                      <button
                        type="submit"
                        className="w-full rounded-full bg-violet-600 py-3.5 text-sm font-bold text-white shadow-lg shadow-violet-200 transition hover:bg-violet-700 focus:outline-none focus:ring-4 focus:ring-violet-200"
                      >
                        Tiếp theo
                      </button>

                      <p className="mt-4 text-center text-xs text-slate-500">
                        Xử lý 100% offline • Export .docx + .xlsx
                      </p>
                    </div>
                  </form>
                </div>
              </div>
            </div>
          </main>
          <LoadingOverlay open={isAnalyzing} />
          {isErrorModalOpen && (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 px-4">
              <div
                className="w-full max-w-sm rounded-2xl bg-white p-6 shadow-xl"
                role="dialog"
                aria-modal="true"
                aria-labelledby="missing-file-title"
              >
                <h2
                  id="missing-file-title"
                  className="text-lg font-semibold text-slate-900"
                >
                  Lỗi
                </h2>
                <p className="mt-2 text-sm text-slate-600">
                  {errorMessage ?? "Đã xảy ra lỗi không xác định."}
                </p>

                <div className="mt-5 flex justify-end">
                    <button
                    type="button"
                    className="rounded-full bg-violet-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-violet-700 focus:outline-none focus:ring-4 focus:ring-violet-200"
                    onClick={() => setIsErrorModalOpen(false)}
                    >
                    Đóng
                    </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
