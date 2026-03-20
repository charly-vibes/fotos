Screen Recording Feature Investigation

## Summary
Investigated adding screen recording to the Fotos app and MCP server. The feature is technically feasible but out of scope for this project.

## Decision: Won't Do
Fotos is a screenshot annotation tool — screen recording is a different product category. Adding it would significantly increase complexity (new GStreamer dependency, pipeline management, audio handling) without serving the core use case.

## Findings (for reference if revisited)

### Recommended approach (if ever pursued)
**ashpd (Screencast portal) + GStreamer (gstreamer-rs + pipewiresrc)**
- Same stack used by Kooha (production Rust GNOME screen recorder on Flathub)
- ashpd already a dependency (used for screenshots) — Screencast API is in the same crate
- All GStreamer plugins (pipewiresrc, openh264enc, vp8enc, opusenc, mp4mux, webmmux) ship in GNOME Platform 48

### Architecture sketch
```
ashpd Screencast portal → PipeWire fd + node_id
  → GStreamer pipeline:
    pipewiresrc fd=X path=Y
      ! videoconvert ! openh264enc/vp8enc
      ! mp4mux/webmmux ! filesink

    (optional audio)
    pulsesrc ! audioconvert ! opusenc ! mux.
```

### New dependencies required
- gstreamer 0.24 (+ gstreamer-video, gstreamer-app)
- Flatpak: add --talk-name=org.freedesktop.portal.ScreenCast to finish-args

### Impact areas
- Backend: new recording/ module, 4+ new Tauri commands, GStreamer pipeline management
- MCP: new tools (start/stop_recording), resources (recordings://), prompts
- Frontend: recording controls, timer, source selection, audio toggles, format settings
- Flatpak: minor manifest change

### Alternatives rejected
- Raw PipeWire + manual encoding: too low-level
- FFmpeg subprocess: can't consume PipeWire stream by node ID
- ffmpeg-next crate: maintenance mode
- scap crate: bypasses portal permission dialog (bad for Flatpak)
- wlr-screencopy: deprecated, GNOME doesn't support it

### Output formats (if pursued)
- WebM (VP8/VP9 + Opus) — open format, good default
- MP4 (OpenH264 + AAC) — widest compatibility (x264 NOT in GNOME runtime)

