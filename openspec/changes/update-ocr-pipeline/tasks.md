## 1. Research & Refactoring
- [x] 1.1 Implement Tesseract wrapper in `ai/ocr.rs` using logic extracted from `commands/ai.rs` (replacing current stub)
- [ ] 1.2 Benchmark current OCR accuracy on complex layouts for baseline

## 2. Image Preprocessing
- [x] 2.1 Implement image upscaling (2x) using `image` crate filters
- [x] 2.2 Add heuristic to trigger upscaling based on estimated font x-height or pixel thresholds (e.g., height < 2000px)

## 3. Tiled OCR Implementation
- [x] 3.1 Implement image tiling logic with configurable overlap (min 100px)
- [x] 3.2 Implement coordinate translation from tile-local to global image space
- [x] 3.3 Parallelize tile processing to ensure performance on multi-core systems
- [x] 3.4 Implement memory-efficient streaming for tiles to prevent OOM on high-res (5K+) images

## 4. Merging & Deduplication
- [x] 4.1 Implement merging algorithm using Intersection over Union (IoU) and string similarity (Levenshtein)
- [x] 4.2 Add logic to prioritize full word detections over clipped words at tile boundaries
- [x] 4.3 Add confidence-based selection for overlapping detections

## 5. Verification & UI
- [x] 5.1 Create unit tests for tiling and coordinate translation
- [ ] 5.2 Validate accuracy improvement on "scrambled" samples
- [x] 5.3 Update UI status messages to reflect multi-stage progress (e.g., "Processing tile 1 of 4...")
