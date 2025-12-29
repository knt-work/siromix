// src/components/FlowNavigation.tsx
import type { FC } from "react";
import { ArrowLeftIcon, ArrowRightIcon, ArrowPathIcon, CheckIcon } from "@heroicons/react/24/outline";

export interface FlowNavigationProps {
  /**
   * Handler for the "Back" button click
   */
  onBack?: () => void;

  /**
   * Handler for the "Next/Submit" button click
   */
  onNext?: () => void;

  /**
   * Label for the Back button (default: "Quay lại")
   */
  backLabel?: string;

  /**
   * Label for the Next button (default: "Tiếp theo")
   */
  nextLabel?: string;

  /**
   * Disable the Next button (e.g., when form is invalid)
   */
  nextDisabled?: boolean;

  /**
   * Disable the Back button
   */
  backDisabled?: boolean;

  /**
   * Show loading state on the Next button
   */
  loading?: boolean;

  /**
   * Subtitle text displayed below the buttons (e.g., "Xử lý 100% offline • Export .docx + .xlsx")
   */
  subtitle?: string;
}

export const FlowNavigation: FC<FlowNavigationProps> = ({
  onBack,
  onNext,
  backLabel = "Quay lại",
  nextLabel = "Tiếp theo",
  nextDisabled = false,
  backDisabled = false,
  loading = false,
  subtitle,
}) => {
  // Determine which icon to use for next button
  const getNextIcon = () => {
    if (loading) return null;
    if (nextLabel.toLowerCase().includes("trộn")) return ArrowPathIcon;
    if (nextLabel.toLowerCase().includes("lưu") || nextLabel.toLowerCase().includes("hoàn tất")) return CheckIcon;
    return ArrowRightIcon;
  };

  const NextIcon = getNextIcon();

  return (
    <div className="pt-2">
      {/* Button Container */}
      <div className="flex items-center justify-between">
        {/* Back Button - only display if onBack is provided */}
        {onBack && (
          <button
            type="button"
            onClick={onBack}
            disabled={backDisabled || loading}
            className="inline-flex items-center gap-2 rounded-full border border-slate-300 bg-white px-6 py-3.5 text-sm font-bold text-slate-700 shadow-sm transition hover:bg-slate-50 focus:outline-none focus:ring-4 focus:ring-slate-200 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <ArrowLeftIcon className="h-5 w-5" />
            <span>{backLabel}</span>
          </button>
        )}

        {/* Spacer when only one button exists */}
        {!onBack && <div></div>}

        {/* Next Button - only display if onNext is provided */}
        {onNext && (
          <button
            type="button"
            onClick={onNext}
            disabled={nextDisabled || loading}
            className="inline-flex items-center gap-2 rounded-full bg-violet-600 px-6 py-3.5 text-sm font-bold text-white shadow-lg shadow-violet-200 transition hover:bg-violet-700 focus:outline-none focus:ring-4 focus:ring-violet-200 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <span>{loading ? "Đang xử lý..." : nextLabel}</span>
            {NextIcon && <NextIcon className="h-5 w-5" />}
          </button>
        )}
      </div>

      {/* Subtitle */}
      {subtitle && (
        <p className="mt-4 text-center text-xs text-slate-500">{subtitle}</p>
      )}
    </div>
  );
};
