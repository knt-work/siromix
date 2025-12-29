// components/ProgressModal.tsx
import type { FC } from "react";
import { CheckIcon } from "@heroicons/react/24/solid";
import { ArrowPathIcon } from "@heroicons/react/24/outline";

export interface ProgressStage {
  label: string;
  done: boolean;
  current?: boolean;
}

export interface ProgressModalProps {
  isOpen: boolean;
  stages: ProgressStage[];
  message?: string;
}

export const ProgressModal: FC<ProgressModalProps> = ({
  isOpen,
  stages,
  message = "Đang xử lý...",
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-slate-900/50 backdrop-blur-sm">
      <div className="relative mx-4 w-full max-w-md rounded-3xl border border-slate-200/60 bg-white shadow-2xl shadow-slate-900/20">
        <div className="px-8 py-10">
          {/* Spinner Icon */}
          <div className="mb-6 flex justify-center">
            <div className="inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-violet-100">
              <ArrowPathIcon className="h-9 w-9 animate-spin text-violet-600" />
            </div>
          </div>

          {/* Title */}
          <h2 className="text-center text-2xl font-bold text-slate-900">
            {message}
          </h2>

          {/* Progress Stages */}
          <div className="mt-8 space-y-3">
            {stages.map((stage, idx) => (
              <div
                key={idx}
                className={`flex items-center gap-3 rounded-xl px-4 py-3 transition ${
                  stage.current
                    ? "bg-violet-50"
                    : stage.done
                    ? "bg-emerald-50"
                    : "bg-slate-50"
                }`}
              >
                {/* Icon */}
                <div
                  className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${
                    stage.done
                      ? "bg-emerald-500"
                      : stage.current
                      ? "bg-violet-500"
                      : "bg-slate-300"
                  }`}
                >
                  {stage.done ? (
                    <CheckIcon className="h-5 w-5 text-white" />
                  ) : (
                    <span className="text-sm font-bold text-white">
                      {idx + 1}
                    </span>
                  )}
                </div>

                {/* Label */}
                <div
                  className={`flex-1 text-sm font-semibold ${
                    stage.current
                      ? "text-violet-700"
                      : stage.done
                      ? "text-emerald-700"
                      : "text-slate-500"
                  }`}
                >
                  {stage.label}
                </div>

                {/* Current indicator */}
                {stage.current && (
                  <div className="h-2 w-2 animate-pulse rounded-full bg-violet-500"></div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
