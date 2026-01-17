// components/shared/SegmentRenderer.tsx
import type { FC } from "react";
import { MathBlock } from "../../lib/mathjax";
import { ommlToMathml } from "../../lib/omml";
import { ImageSegment } from "../ImageSegment";
import type { Segment } from "../../store/mixStore";

interface SegmentRendererProps {
  segment: Segment;
  index: number;
  className?: string;
}

/**
 * Shared component for rendering different segment types (Text, Math, Image)
 * Extracted from PreviewPage to avoid duplication
 */
export const SegmentRenderer: FC<SegmentRendererProps> = ({
  segment,
  index,
  className = "",
}) => {
  switch (segment.type) {
    case "Text":
      return (
        <span key={index} className={className}>
          {segment.text}{" "}
        </span>
      );

    case "Image":
      return (
        <ImageSegment
          key={index}
          assetPath={segment.asset_path}
          className={className}
        />
      );

    case "Math": {
      const mathml = ommlToMathml(segment.omml);
      if (!mathml || mathml.trim() === "") {
        console.error(
          "Failed to convert OMML to MathML:",
          segment.omml.substring(0, 100)
        );
        return (
          <span key={index} className="italic text-red-500">
            [Math]
          </span>
        );
      }
      return (
        <MathBlock
          key={index}
          mathml={mathml}
          className={`mx-1 inline-block align-middle ${className}`}
        />
      );
    }

    default:
      return null;
  }
};

/**
 * Helper to render an array of segments
 */
export const renderSegments = (segments: Segment[], className?: string) => {
  return segments.map((seg, idx) => (
    <SegmentRenderer key={idx} segment={seg} index={idx} className={className} />
  ));
};
