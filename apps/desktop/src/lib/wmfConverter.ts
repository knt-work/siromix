// lib/wmfConverter.ts

/**
 * WMF/EMF Converter for Tauri Desktop App
 * 
 * This module provides utilities to detect and handle WMF/EMF image files.
 * 
 * IMPORTANT NOTE:
 * JavaScript-based WMF parsers (wmf, rtf.js) have very limited support
 * for the complex WMF format, especially old Equation Editor 3.0 WMF files.
 * 
 * Current approach: Detection only - conversion will be added in future
 * using either:
 * - Native Rust WMF renderer (bundle ImageMagick or similar)
 * - Server-side conversion
 * - Better JS library when available
 */

/**
 * Check if file extension is WMF or EMF
 */
export function isWmfFile(path: string): boolean {
  const ext = path.toLowerCase().split(".").pop();
  return ext === "wmf" || ext === "emf";
}

/**
 * Get information about WMF file
 */
export function getWmfInfo(path: string): { filename: string; ext: string } {
  const parts = path.split(/[/\\]/);
  const filename = parts[parts.length - 1] || "unknown";
  const ext = filename.split(".").pop()?.toUpperCase() || "WMF";
  return { filename, ext };
}
