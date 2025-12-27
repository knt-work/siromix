import React, { useEffect, useRef } from "react";
import "mathjax/es5/tex-mml-svg.js";

declare global {
  interface Window {
    MathJax?: {
      typesetPromise?: (elements: HTMLElement[]) => Promise<void>;
    };
  }
}

type MathBlockProps = {
  mathml?: string;
  tex?: string;
  className?: string;
};

export const MathBlock: React.FC<MathBlockProps> = ({
  mathml,
  tex,
  className,
}) => {
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const content = mathml ?? tex ?? "";
    if (!content) return;

    el.innerHTML = content;

    if (window.MathJax?.typesetPromise) {
      window.MathJax.typesetPromise([el]).catch(() => {
        // ignore MathJax errors in UI
      });
    }
  }, [mathml, tex]);

  return React.createElement("div", {
    ref: containerRef,
    className,
  });
};
