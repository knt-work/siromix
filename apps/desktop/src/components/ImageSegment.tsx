// components/ImageSegment.tsx
import React, { type FC } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { isWmfFile, getWmfInfo } from "../lib/wmfConverter";

interface ImageSegmentProps {
  assetPath: string;
  className?: string;
}

/**
 * Renders an image segment with WMF/EMF detection
 * 
 * For normal images (PNG, JPG, etc.): renders directly via convertFileSrc
 * For WMF/EMF files: 
 *   - If backend converted to PNG → will receive .png path, renders normally
 *   - If not converted → displays fallback message with instructions
 * 
 * Backend (Rust) attempts automatic WMF→PNG conversion using ImageMagick.
 * This component handles the fallback case when ImageMagick is not available.
 */
export const ImageSegment: FC<ImageSegmentProps> = ({ assetPath, className }) => {
  const [imageStyle, setImageStyle] = React.useState<React.CSSProperties>({ 
    height: "auto", 
    maxWidth: "100%" 
  });

  // Debug log
  console.log("[ImageSegment] Rendering:", assetPath);
  
  // Check if this is an unconverted WMF/EMF file
  // (Backend should have converted these to PNG, but fallback if not)
  if (isWmfFile(assetPath)) {
    console.warn("[ImageSegment] WMF file not converted:", assetPath);
    const { filename, ext } = getWmfInfo(assetPath);
    
    return (
      <div
        className={`my-2 rounded-md border border-amber-200 bg-amber-50 ${className || ""}`}
        style={{ minHeight: "80px" }}
      >
        <div className="flex flex-col gap-2 p-3">
          <div className="flex items-center gap-2">
            <svg
              className="mt-0.5 h-5 w-5 shrink-0 text-amber-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
            <div className="flex-1">
              <p className="text-sm font-medium text-amber-900">
                Không hiển thị được công thức {ext}
              </p>
              <p className="mt-1 text-xs text-amber-700">
                File <code className="rounded bg-amber-100 px-1 py-0.5 font-mono text-xs">{filename}</code>{" "}
                sử dụng định dạng {ext} cũ (từ Equation Editor 3.0) không được trình duyệt web hỗ trợ.
              </p>
              <p className="mt-2 text-xs text-amber-700">
                <strong>⚙️ Yêu cầu ImageMagick:</strong> Để tự động chuyển đổi WMF sang PNG, 
                cần cài đặt ImageMagick trên máy.
              </p>
              <div className="mt-3 space-y-1.5 text-xs text-amber-800">
                <p className="font-semibold">💡 Hướng dẫn khắc phục:</p>
                <ol className="ml-4 list-decimal space-y-1">
                  <li>
                    <strong>Option 1:</strong> Cài{" "}
                    <a 
                      href="https://imagemagick.org/script/download.php" 
                      target="_blank" 
                      rel="noopener noreferrer"
                      className="text-violet-700 underline hover:text-violet-800"
                    >
                      ImageMagick
                    </a>
                    {" "}→ Restart app → Import lại file.
                  </li>
                  <li>
                    <strong>Option 2:</strong> Mở file Word, chọn công thức cũ → Copy → Paste Special → 
                    chọn "Picture (PNG)".
                  </li>
                  <li>
                    <strong>Option 3:</strong> Sử dụng Insert → Equation (Math) để nhập lại công thức bằng công cụ mới của Word.
                  </li>
                </ol>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Normal image format - render directly
  console.log("[ImageSegment] Rendering normal image:", assetPath);
  
  // Use 'asset' protocol instead of 'stream' for better compatibility with dynamic files
  const imageSrc = convertFileSrc(assetPath, "asset");
  
  const handleImageLoad = (e: React.SyntheticEvent<HTMLImageElement>) => {
    const img = e.currentTarget;
    const aspectRatio = img.naturalWidth / img.naturalHeight;
    
    // Landscape (wide) → 24px, Square/Portrait → 75px
    const targetHeight = aspectRatio > 1.0 ? 24 : 75;
    
    setImageStyle({
      height: `${targetHeight}px`,
      width: "auto"
    });
    
    console.log(`[ImageSegment] Aspect ratio: ${aspectRatio.toFixed(2)}, height: ${targetHeight}px`);
  };
  
  return (
    <img
      src={imageSrc}
      alt="Ảnh câu hỏi"
      className={`inline-block align-middle rounded border border-slate-200 bg-white ${className || ""}`}
      style={imageStyle}
      onLoad={handleImageLoad}
      onError={() => {
        console.error("[ImageSegment] Image load error for:", assetPath);
        console.error("[ImageSegment] Attempted URL:", imageSrc);
      }}
    />
  );
};
