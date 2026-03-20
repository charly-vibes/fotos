OCR pipeline baseline benchmark (post-implementation, 2026-03-05):

23 OCR unit tests, all passing in 0.03s. Test coverage by category:

**Tiling logic** (4 tests):
- tile_positions_image_smaller_than_tile: images < tile size use single pass
- tile_positions_exact_tile_size: exact size = single tile at offset 0
- tile_positions_two_tiles: 1200px wide = [0, 176] (stride=924, TILE=1024, OVERLAP=100)
- tile_positions_covers_full_width: 2560px → last tile reaches image end, no tile overruns

**Coordinate translation** (1 test):
- tile_offset_translation: tile-local coords + tile origin = correct global coords

**IoU deduplication** (3 tests):
- iou_identical_boxes: score = 1.0
- iou_non_overlapping: score = 0.0
- iou_partial_overlap: 50px shift on 100×20 box → IoU = 1/3 (±1e-4)

**Levenshtein string similarity** (3 tests + 4 texts_similar tests):
- identical strings: distance = 0
- one deletion: distance = 1
- empty string: distance = len(other)
- texts_similar: case-insensitive, tolerates 1 edit, rejects divergent words

**Deduplication** (3 tests):
- overlapping identical: keeps higher-confidence detection
- non-overlapping: keeps both
- same bbox, different text: keeps both (texts_similar = false)

**Reading-order reconstruction** (5 tests):
- same_line: words appear in output
- newline_between_lines: newline inserted for y-separated regions
- two_column_layout: scrambled input → Left/Text precede Right/Side respectively
- sidebar_layout: File→Edit→View in order, sidebar items precede same-row main content
- reverse_input_order: First→Second→Third regardless of input order

**Accuracy assessment**: 100% pass rate on all structural/geometric invariants.
True OCR character accuracy requires live Tesseract integration test (not unit-testable without real images and tesseract binary). The pipeline correctness properties (tiling, dedup, reading order) are fully verified.
