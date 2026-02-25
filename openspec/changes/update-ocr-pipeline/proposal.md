# Change: Update OCR Pipeline with Tiling and Preprocessing

## Why
The current single-pass OCR often fails on complex layouts (multi-window, sidebars, columns) by scrambling the reading order. High-resolution screenshots also suffer from suboptimal DPI representation in Tesseract; upscaling helps Tesseract recognize small characters more reliably by providing more pixels per glyph.

## What Changes
- Implement image upscaling preprocessing to improve OCR accuracy on small text.
- Implement a tiled OCR strategy for large/complex images to maintain layout integrity.
- Implement word-level deduplication and merging for overlapping tiles using spatial and textual heuristics.
- Add heuristics for selecting optimal Tesseract Page Segmentation Modes (PSM).

## Impact
- Affected specs: `specs/ai-processing/spec.md`
- Affected code: `src-tauri/src/ai/ocr.rs`, `src-tauri/src/commands/ai.rs`
- Dependencies: Utilizes existing `image` crate; may require `rayon` or `tokio` for efficient parallel tile processing without blocking the main thread.
