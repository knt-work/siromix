import type { FC } from "react";

export interface LoadingOverlayProps {
  open: boolean;
}

export const LoadingOverlay: FC<LoadingOverlayProps> = ({ open }) => {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 backdrop-blur-sm">
      <div className="flex flex-col items-center gap-4 rounded-2xl bg-white/90 px-8 py-6 shadow-xl shadow-slate-900/10">
        <div className="h-10 w-10 animate-spin rounded-full border-4 border-violet-200 border-t-violet-600" aria-hidden="true" />
        <p className="text-sm font-medium text-slate-800">Đang phân tích đề…</p>
      </div>
    </div>
  );
};
