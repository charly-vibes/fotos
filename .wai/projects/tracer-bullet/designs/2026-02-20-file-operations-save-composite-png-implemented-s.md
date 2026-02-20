File Operations: Save Composite PNG

Implemented save_image command to composite annotations onto screenshots and save as PNG files.

## Design Decisions:

1. **Compositing Strategy**: Clone base image to RGBA8, then composite annotations on top using imageproc. Avoids modifying the original cached image.

2. **Rectangle Drawing**: Use imageproc::draw_hollow_rect_mut with multiple passes to simulate stroke width. Each stroke_width unit draws one additional outline rect.

3. **Color Parsing**: Parse #RRGGBB hex colors to RGBA. Default to red (#FF0000) if invalid.

4. **Default Path Generation**:
   - Format: ~/Pictures/Fotos/fotos-YYYYMMDD-HHMMSS.png
   - Uses directories crate for cross-platform Pictures dir
   - Uses chrono::Local for timestamp
   - Creates Fotos directory if needed

5. **Tilde Expansion**: Manual implementation since shellexpand isn't a dependency. Handles ~/path by replacing with home directory from UserDirs.

6. **Error Handling**: Descriptive error messages for UUID parsing, image lookup, directory creation, and save failures.

7. **Frontend Integration**: Ctrl+S triggers save with empty path (auto-generates default). Status bar shows saved path or error.

## Implementation:
- save_image command in commands/files.rs
- Helper functions: composite_rectangle, parse_color, generate_default_path, expand_tilde
- Ctrl+S keyboard shortcut in app.js
- Uses ImageStore for image lookup (injected via Tauri state)

Ready for smoke testing.
