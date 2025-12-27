import type { FC } from "react";
import { useParams } from "react-router-dom";

export const PreviewPage: FC = () => {
  const { jobId } = useParams<{ jobId: string }>();

  return (
    <div className="min-h-screen bg-slate-50 px-6 py-10">
      <div className="mx-auto max-w-5xl">
        <h1 className="text-3xl font-bold tracking-tight text-slate-900">
          Xem trước đề đã nhập
        </h1>
        <p className="mt-4 text-sm text-slate-600">
          Job ID: <span className="font-mono text-slate-800">{jobId}</span>
        </p>
      </div>
    </div>
  );
};
