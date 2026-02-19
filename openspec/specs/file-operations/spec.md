# File Operations

Capability for saving, exporting, loading, and clipboard operations on screenshots and annotations in Fotos (`io.github.charly.fotos`).

## Purpose

This capability covers the Rust Tauri commands `save_image`, `copy_to_clipboard`, and `export_annotations` (section 8 of the root spec), as well as the configurable save directory, default format, JPEG quality, and auto-copy behavior defined in the settings schema (section 11).

## Requirements

### Requirement: Save Image

The system SHALL composite the base screenshot image with all committed annotations and save the result to the specified file path. The `save_image` Tauri command MUST accept an `image_id` (String), an `annotations` array (`Vec<Annotation>`), a `format` string, and a `path` string. The command MUST support the formats `png`, `jpg`, and `webp`. The compositing MUST render annotations at the original image dimensions (no viewport transform applied). On success the command SHALL return the absolute path of the written file as a String.

#### Scenario: Save as PNG
- **WHEN** the frontend invokes `save_image` with `format: "png"` and a valid `path`
- **THEN** the backend SHALL render the screenshot with all annotations composited onto it and write a PNG file to the given path
- **THEN** the command SHALL return the absolute path of the saved file

#### Scenario: Save as JPEG with configurable quality
- **WHEN** the frontend invokes `save_image` with `format: "jpg"` and a valid `path`
- **THEN** the backend SHALL encode the composited image as JPEG using the `jpegQuality` value from user settings (default 90, range 1-100)
- **THEN** the resulting file SHALL be written to the given path

#### Scenario: Save as WebP
- **WHEN** the frontend invokes `save_image` with `format: "webp"` and a valid `path`
- **THEN** the backend SHALL encode the composited image as WebP and write it to the given path

#### Scenario: Invalid image ID
- **WHEN** the frontend invokes `save_image` with an `image_id` that does not correspond to any captured screenshot in memory
- **THEN** the command SHALL return an error string describing the unknown image ID

#### Scenario: Invalid or inaccessible path
- **WHEN** the frontend invokes `save_image` with a `path` whose parent directory does not exist or is not writable
- **THEN** the command SHALL return an error string describing the file system failure

---

### Requirement: Save As

The system SHALL allow the user to choose a destination file path via a native save dialog before saving. The frontend MUST use the Tauri dialog plugin (`tauri-plugin-dialog`) to present a file-picker dialog that filters by supported image formats (PNG, JPG, WebP). When the user confirms a path, the frontend SHALL invoke `save_image` with the chosen path and the format inferred from the file extension. If the user cancels the dialog, no save operation SHALL occur.

#### Scenario: User picks destination and saves
- **WHEN** the user triggers Save As (Ctrl+Shift+S or toolbar button)
- **THEN** the system SHALL open a native save dialog with file type filters for PNG, JPG, and WebP
- **THEN** upon user confirmation the system SHALL invoke `save_image` with the selected path and the format matching the chosen file extension

#### Scenario: User cancels save dialog
- **WHEN** the user triggers Save As and then cancels the native save dialog
- **THEN** no file SHALL be written and no error SHALL be raised

---

### Requirement: Copy to Clipboard

The system SHALL composite the base screenshot image with all committed annotations and place the resulting image on the system clipboard. The `copy_to_clipboard` Tauri command MUST accept an `image_id` (String) and an `annotations` array (`Vec<Annotation>`). The composited image SHALL be placed on the clipboard as PNG image data using the `tauri-plugin-clipboard-manager` plugin. On success the command SHALL return `Ok(())`.

#### Scenario: Copy composited image to clipboard
- **WHEN** the frontend invokes `copy_to_clipboard` with a valid `image_id` and annotations
- **THEN** the backend SHALL composite the screenshot with annotations and write the resulting PNG image data to the system clipboard
- **THEN** the image SHALL be pasteable in any application that accepts image clipboard content

#### Scenario: Copy with no annotations
- **WHEN** the frontend invokes `copy_to_clipboard` with a valid `image_id` and an empty annotations array
- **THEN** the backend SHALL place the unmodified screenshot image on the clipboard

#### Scenario: Invalid image ID on copy
- **WHEN** the frontend invokes `copy_to_clipboard` with an `image_id` that does not correspond to any captured screenshot in memory
- **THEN** the command SHALL return an error string describing the unknown image ID

---

### Requirement: Export Annotations

The system SHALL export the annotation data for a given screenshot as a JSON string suitable for reimport. The `export_annotations` Tauri command MUST accept an `image_id` (String) and an `annotations` array (`Vec<Annotation>`). The returned JSON MUST contain the full annotation array with all properties as defined in the annotation data model (id, type, geometry, style, type-specific fields, metadata). The JSON output MUST be a valid, serialized representation that can be deserialized back into the same annotation structure.

#### Scenario: Export annotations as JSON
- **WHEN** the frontend invokes `export_annotations` with a valid `image_id` and a non-empty annotations array
- **THEN** the command SHALL return a JSON string containing all annotation objects with their complete properties
- **THEN** the returned JSON SHALL be valid and parseable

#### Scenario: Export with no annotations
- **WHEN** the frontend invokes `export_annotations` with a valid `image_id` and an empty annotations array
- **THEN** the command SHALL return a JSON string representing an empty array (`[]`)

#### Scenario: Round-trip fidelity
- **WHEN** the exported JSON is deserialized and used to reconstruct annotations
- **THEN** the reconstructed annotations MUST be identical in structure and values to the original annotations that were exported

---

### Requirement: Load Image

The system SHALL allow the user to load an existing image file from disk for annotation. The frontend MUST use the Tauri dialog plugin to present a native file-open dialog filtering for supported image formats (PNG, JPG, WebP). Upon selection, the Rust backend SHALL load the image via the `image` crate, assign it a new UUID, store it in the in-memory screenshot registry, and emit a `screenshot-ready` event to the frontend. The frontend SHALL then display the loaded image on the base canvas layer and enable the annotation toolbar.

#### Scenario: Load a PNG file for annotation
- **WHEN** the user opens a PNG file via the load image dialog
- **THEN** the backend SHALL decode the image, assign a UUID, store it in memory, and emit a `screenshot-ready` event
- **THEN** the frontend SHALL display the image on the canvas and activate annotation tools

#### Scenario: Load a JPEG file
- **WHEN** the user opens a JPEG file via the load image dialog
- **THEN** the backend SHALL decode and load the image with the same behavior as for PNG

#### Scenario: Load a WebP file
- **WHEN** the user opens a WebP file via the load image dialog
- **THEN** the backend SHALL decode and load the image with the same behavior as for PNG

#### Scenario: User cancels load dialog
- **WHEN** the user opens the load image dialog and cancels without selecting a file
- **THEN** no image SHALL be loaded and the current canvas state SHALL remain unchanged

#### Scenario: Unsupported or corrupt file
- **WHEN** the user selects a file that cannot be decoded as a supported image format
- **THEN** the system SHALL display an error message to the user and the canvas state SHALL remain unchanged

---

### Requirement: Default Save Directory

The system SHALL provide a configurable default save directory for screenshot files. The default value MUST be `~/Pictures/Fotos` (resolved via the `directories` crate to the platform-appropriate pictures directory). The setting SHALL be stored under the `capture.saveDirectory` key in the user settings (managed by `tauri-plugin-store`). When the user triggers a quick save (Ctrl+S) without having previously used Save As for the current image, the system SHALL save to the default directory using an auto-generated filename based on the timestamp and the configured default format.

#### Scenario: First save uses default directory
- **WHEN** the user triggers Save (Ctrl+S) for a newly captured screenshot that has not been saved before
- **THEN** the system SHALL save the file to the configured default save directory
- **THEN** the filename SHALL be auto-generated using the capture timestamp (e.g., `fotos-2025-01-15-143022.png`)
- **THEN** the file format SHALL match the `capture.defaultFormat` setting (default `png`)

#### Scenario: Default directory does not exist
- **WHEN** the user triggers Save and the configured default save directory does not exist
- **THEN** the system SHALL create the directory (including intermediate directories) before writing the file

#### Scenario: Custom save directory configured
- **WHEN** the user has changed `capture.saveDirectory` to a custom path in settings
- **THEN** subsequent quick saves SHALL write files to the custom directory instead of `~/Pictures/Fotos`

---

### Requirement: Auto-copy After Capture

The system SHALL optionally copy the captured screenshot to the system clipboard immediately after capture completes, before any annotation. This behavior is controlled by the `capture.copyToClipboardAfterCapture` boolean setting (default `true`). When enabled, the raw screenshot image (without annotations) SHALL be placed on the clipboard as PNG data immediately after the capture result is received.

#### Scenario: Auto-copy enabled (default)
- **WHEN** a screenshot capture completes and `capture.copyToClipboardAfterCapture` is `true`
- **THEN** the system SHALL immediately copy the raw captured image (without annotations) to the system clipboard as PNG data
- **THEN** this copy SHALL occur before the user begins any annotation

#### Scenario: Auto-copy disabled
- **WHEN** a screenshot capture completes and `capture.copyToClipboardAfterCapture` is `false`
- **THEN** the system SHALL NOT automatically copy the image to the clipboard

#### Scenario: Auto-copy does not block annotation
- **WHEN** auto-copy is enabled and the clipboard write is in progress
- **THEN** the frontend SHALL NOT be blocked from displaying the image and enabling annotation tools
