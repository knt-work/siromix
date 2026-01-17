// components/ErrorBoundary.tsx
import React, { Component, type ReactNode } from "react";
import { XCircleIcon, ArrowPathIcon } from "@heroicons/react/24/outline";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

/**
 * Error Boundary component to catch and handle React errors gracefully
 * Prevents the entire app from crashing when a component throws an error
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    // Update state so the next render will show the fallback UI
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // Log error to console for debugging
    console.error("ErrorBoundary caught an error:", error, errorInfo);
    
    // Store error info in state
    this.setState({
      error,
      errorInfo,
    });

    // TODO: Send error to logging service (e.g., Sentry)
    // logErrorToService(error, errorInfo);
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      // Custom fallback UI
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default fallback UI
      return (
        <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-red-50 to-orange-50 p-6">
          <div className="w-full max-w-2xl rounded-3xl border border-red-200 bg-white p-8 shadow-2xl">
            {/* Error Icon */}
            <div className="mb-6 flex justify-center">
              <div className="inline-flex h-16 w-16 items-center justify-center rounded-full bg-red-100">
                <XCircleIcon className="h-10 w-10 text-red-600" />
              </div>
            </div>

            {/* Error Title */}
            <h1 className="mb-4 text-center text-3xl font-bold text-slate-900">
              Oops! Có lỗi xảy ra
            </h1>

            {/* Error Description */}
            <p className="mb-6 text-center text-slate-600">
              Ứng dụng gặp lỗi không mong muốn. Vui lòng thử lại hoặc liên hệ hỗ trợ nếu lỗi tiếp tục xảy ra.
            </p>

            {/* Error Details (dev mode) */}
            {process.env.NODE_ENV === "development" && this.state.error && (
              <div className="mb-6 rounded-2xl bg-red-50 p-4">
                <h2 className="mb-2 text-sm font-semibold text-red-800">
                  Chi tiết lỗi:
                </h2>
                <pre className="overflow-auto text-xs text-red-700">
                  {this.state.error.toString()}
                </pre>
                {this.state.errorInfo && (
                  <details className="mt-4">
                    <summary className="cursor-pointer text-xs font-medium text-red-800">
                      Component Stack
                    </summary>
                    <pre className="mt-2 overflow-auto text-xs text-red-600">
                      {this.state.errorInfo.componentStack}
                    </pre>
                  </details>
                )}
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex items-center justify-center gap-3">
              <button
                onClick={this.handleReset}
                className="inline-flex items-center gap-2 rounded-full bg-violet-600 px-6 py-3 text-sm font-bold text-white shadow-lg shadow-violet-200 transition hover:bg-violet-700 focus:outline-none focus:ring-4 focus:ring-violet-200"
              >
                <ArrowPathIcon className="h-5 w-5" />
                <span>Thử lại</span>
              </button>

              <button
                onClick={() => window.location.reload()}
                className="inline-flex items-center gap-2 rounded-full border border-slate-300 bg-white px-6 py-3 text-sm font-bold text-slate-700 shadow-sm transition hover:bg-slate-50 focus:outline-none focus:ring-4 focus:ring-slate-200"
              >
                <span>Tải lại trang</span>
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
