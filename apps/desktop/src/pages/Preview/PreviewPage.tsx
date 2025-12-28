import type { FC } from "react";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { MathBlock } from "../../lib/mathjax";
import { ommlToMathml } from "../../lib/omml";

type Segment =
  | { type: "Text"; text: string }
  | { type: "Image"; asset_path: string }
  | { type: "Math"; omml: string };

type OptionItem = {
  label: string;
  locked: boolean;
  content: Segment[];
};

type Question = {
  number: number;
  stem: Segment[];
  options: OptionItem[];
  correct_label: string;
};

type ParsedDoc = {
  questions: Question[];
};

export const PreviewPage: FC = () => {
  const { jobId } = useParams<{ jobId: string }>();

  const [parsed, setParsed] = useState<ParsedDoc | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!jobId) return;

    let isCancelled = false;
    setLoading(true);
    setError(null);

    invoke<ParsedDoc>("get_parsed", { jobId })
      .then((doc) => {
        if (isCancelled) return;
        console.log("Parsed doc:", doc);
        console.log("Number of questions:", doc.questions?.length);
        setParsed(doc);
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
  }, [jobId]);

  const renderSegment = (segment: Segment, key: number) => {
    switch (segment.type) {
      case "Text":
        return <span key={key}>{segment.text} </span>;
      case "Image":
        return (
          <img
            key={key}
            src={convertFileSrc(segment.asset_path, "stream")}
            alt="Ảnh câu hỏi"
            className="my-2 max-w-full rounded-md border border-slate-200 bg-white object-contain"
            style={{ height: "auto" }}
          />
        );
      case "Math": {
        const mathml = ommlToMathml(segment.omml);
        if (!mathml || mathml.trim() === '') {
          console.error("Failed to convert OMML to MathML:", segment.omml.substring(0, 100));
          return <span key={key} className="text-red-500 italic">[Math]</span>;
        }
        return (
          <MathBlock
            key={key}
            mathml={mathml}
            className="mx-1 inline-block align-middle"
          />
        );
      }
      default:
        return null;
    }
  };

  return (
    <div className="min-h-screen bg-slate-50 px-6 py-10">
      <div className="mx-auto max-w-5xl">
        <h1 className="text-3xl font-bold tracking-tight text-slate-900">
          Xem trước đề đã nhập
        </h1>
        <p className="mt-4 text-sm text-slate-600">
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
                      {q.stem.map((seg, idx) => renderSegment(seg, idx))}
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
                            isCorrect ? "bg-emerald-50 font-semibold text-emerald-800" : ""
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
                              opt.content.map((seg, idx) =>
                                renderSegment(seg, idx),
                              )
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
                  Không tìm thấy câu hỏi nào trong file. Parsed doc: {JSON.stringify(parsed)}
                </p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
