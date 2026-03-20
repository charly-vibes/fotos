Settings with Hardcoded Defaults

Implemented Default trait for all settings structs to provide sensible hardcoded defaults for the tracer-bullet phase.

## Design Decisions:

1. **Manual Default Implementation**: Used manual impl Default instead of derive to have explicit control over each default value and better documentation.

2. **Default Values**:
   - Capture: fullscreen mode, PNG format, clipboard copy enabled, no mouse cursor, no delay
   - Annotation: red stroke (#FF0000), 2px width, sans-serif font, 16px size
   - AI: English OCR, Claude as default provider, localhost Ollama
   - UI: system theme, AI panel and status bar visible

3. **No-Op Setters**: set_settings() and set_api_key() are no-ops for tracer phase. This allows the frontend to call these commands without errors, but changes won't persist.

4. **Model Defaults**: Chose current/recent model IDs:
   - Claude: claude-sonnet-4-5
   - OpenAI: gpt-4o
   - Gemini: gemini-2.0-flash-exp
   - Ollama: llama3.2-vision

5. **Path Convention**: Using ~/Pictures/Fotos for save directory (will need expansion in full implementation).

## Implementation:
- Default trait impl for CaptureSettings, AnnotationSettings, AiSettings, UiSettings, Settings
- get_settings() returns Settings::default()
- set_settings() and set_api_key() are no-ops (return Ok)

Ready for file operations and smoke tests.
