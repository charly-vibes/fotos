## MODIFIED Requirements

### Requirement: OCR Text Extraction via Tesseract
The system SHALL extract text from screenshot images using Tesseract OCR. The OCR engine SHALL return both the full extracted text and per-word bounding boxes with confidence scores.

To improve accuracy on complex layouts or high-resolution images, the system SHALL:
1.  **Upscale** the image (or specific regions) if the resolution or font density is below optimal OCR thresholds (e.g., if the image height is less than 2000px).
2.  **Tile** large images into overlapping blocks (min 100px overlap), processing each block independently to maintain layout integrity. 
3.  **Merge** results from multiple tiles using a deduplication algorithm that combines spatial overlap (IoU > 0.5) and string similarity (Levenshtein distance).
4.  **Prioritize Integrity**: If a word is clipped by a tile boundary (and not within the overlap zone), the system SHALL prefer the detection from the adjacent tile where the word is fully contained.
5.  **Translate** tile-local coordinates back to the original image coordinate space.
6.  **Memory Management**: For extremely high-resolution images (e.g., > 5000px width/height), the system SHALL process tiles in a memory-efficient sequence to avoid exhaustion.

The system SHALL use PSM mode 3 (fully automatic) for full-image passes and MAY use PSM mode 6 (single uniform block) for individual tiles.

#### Scenario: Extract text from complex multi-column layout
- **WHEN** the user invokes OCR on a screenshot with multiple distinct text columns
- **THEN** the system SHALL divide the image into overlapping tiles
- **THEN** the system SHALL run OCR on each tile independently
- **THEN** the system SHALL merge the overlapping detections and return a single, correctly ordered `OcrResult`

#### Scenario: Extract text from high-resolution screenshot
- **WHEN** the user invokes OCR on a high-DPI screenshot with small text
- **THEN** the system SHALL upscale the image before processing (unless the image is already at maximum memory capacity)
- **THEN** the system SHALL return an `OcrResult` with bounding boxes mapped back to the original (non-upscaled) dimensions

#### Scenario: Deduplicate overlapping words
- **WHEN** a word is detected in two overlapping tiles
- **THEN** the system SHALL use Intersection over Union (IoU) and string comparison to identify the duplication
- **THEN** the system SHALL keep the detection with the highest confidence score or the one that is not clipped by a boundary

#### Scenario: Handle very large images
- **WHEN** the user captures a 5K+ resolution multi-monitor setup
- **THEN** the system SHALL process the image in tiles without upscaling the entire canvas simultaneously
- **THEN** the system SHALL yield progress updates to the UI for each processed tile
