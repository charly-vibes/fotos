# Capability: UI Shell

## Purpose

The UI shell provides the top-level layout, toolbar, panels, dialogs, status bar, keyboard shortcuts, and theme support for Fotos (io.github.charly.fotos). It is the app's chrome surrounding the annotation canvas. Built with vanilla HTML/CSS/JS and ES modules, no web frameworks.

## Requirements

### Requirement: Compact Icon-Only Toolbar

The toolbar SHALL render as a horizontal `<header id="toolbar">` at the top of the window containing icon-only buttons grouped by function with visual separators between groups.

Tool groups and their order (left to right):

1. **Capture tools** (`#capture-tools`) -- Capture Region, Capture Fullscreen, Capture Window
2. **Annotation tools** (`#annotation-tools`) -- Arrow, Rectangle, Ellipse, Text, Blur, Step Number, Freehand, Highlight, Crop, Select
3. **Style controls** (`#style-controls`) -- Color picker trigger, Size picker
4. **History controls** (`#history-controls`) -- Undo, Redo
5. *(spacer)*
6. **AI tools** (`#ai-tools`) -- Extract Text (OCR), Auto-Blur PII, AI Analyze
7. **Output tools** (`#output-tools`) -- Copy to Clipboard, Save, Save As

Each group SHALL be wrapped in a `<div class="tool-group">` element. A `<div class="separator">` SHALL appear between adjacent groups. A `<div class="spacer">` SHALL push AI and output groups to the right.

#### Scenario: Toolbar renders all groups on app load
- **WHEN** the app window becomes visible
- **THEN** the toolbar MUST display all seven tool groups in the specified order with separators between them

#### Scenario: Buttons are icon-only with tooltips
- **WHEN** the user hovers over any toolbar button
- **THEN** a tooltip MUST appear showing the tool name and its keyboard shortcut (e.g., "Arrow (A)", "Undo (Ctrl+Z)")

### Requirement: Active Tool Visual Indicator

The toolbar SHALL visually highlight the currently active annotation tool button with a distinct style (e.g., `aria-pressed="true"` and an accent-colored background).

#### Scenario: Selecting a tool highlights its button
- **WHEN** the user clicks an annotation tool button or presses its keyboard shortcut
- **THEN** the corresponding button MUST receive the active visual indicator
- **THEN** any previously active tool button MUST lose the active indicator

### Requirement: Annotation Tool Buttons

Each annotation tool button SHALL carry a `data-tool` attribute matching its tool identifier: `arrow`, `rect`, `ellipse`, `text`, `blur`, `step`, `freehand`, `highlight`, `crop`, `select`.

#### Scenario: Clicking a tool button activates that tool
- **WHEN** the user clicks a button with `data-tool="rect"`
- **THEN** the app state `activeTool` MUST be set to `"rect"`

### Requirement: Action Buttons

Capture, history, AI, and output buttons SHALL carry a `data-action` attribute (e.g., `capture-region`, `undo`, `redo`, `ocr`, `auto-blur`, `ai-analyze`, `copy-clipboard`, `save`, `save-as`). Clicking an action button SHALL invoke the corresponding command.

#### Scenario: Clicking the undo button performs undo
- **WHEN** the user clicks the button with `data-action="undo"`
- **THEN** the most recent annotation command MUST be undone

#### Scenario: Clicking save invokes the save command
- **WHEN** the user clicks the button with `data-action="save"`
- **THEN** the save workflow MUST be triggered (equivalent to Ctrl+S)

---

### Requirement: Color and Opacity Picker

The style controls group SHALL include a color picker that allows the user to set the annotation stroke color, fill color, and opacity. The picker MUST be accessible from the `#color-picker-trigger` element in the toolbar.

#### Scenario: Opening the color picker
- **WHEN** the user clicks the color picker trigger in the toolbar
- **THEN** a color picker popover MUST appear showing hue/saturation selection, a brightness/lightness slider, an opacity slider, and a hex/RGB input field

#### Scenario: Changing stroke color
- **WHEN** the user selects a new color in the color picker
- **THEN** the app state `strokeColor` MUST update to the chosen color value
- **THEN** subsequent annotations MUST use the new stroke color

#### Scenario: Changing opacity
- **WHEN** the user adjusts the opacity slider
- **THEN** the app state `opacity` MUST update to the chosen value (0.0 to 1.0)
- **THEN** subsequent annotations MUST render at the new opacity

### Requirement: Recent Colors

The color picker SHALL display a row of recently used colors for quick reselection.

#### Scenario: Recent color appears after use
- **WHEN** the user selects a color and draws an annotation
- **THEN** that color MUST appear in the recent colors row of the color picker

---

### Requirement: Stroke Width Control

The style controls group SHALL include a stroke width picker (`#size-picker`) that allows the user to set the stroke width for annotation drawing tools. The default stroke width SHALL be 2 pixels.

#### Scenario: Changing stroke width
- **WHEN** the user selects a new stroke width value
- **THEN** the app state `strokeWidth` MUST update to the chosen value
- **THEN** subsequent annotations MUST use the new stroke width

### Requirement: Font Size Control

The size picker SHALL include a font size control for the text annotation tool. The default font size SHALL be 16 pixels.

#### Scenario: Changing font size
- **WHEN** the user adjusts the font size control
- **THEN** the app state `fontSize` MUST update to the chosen value
- **THEN** subsequent text annotations MUST render at the new font size

#### Scenario: Font size control visibility
- **WHEN** the active tool is `text` or `step`
- **THEN** the font size control MUST be visible in the size picker
- **WHEN** the active tool is not `text` or `step`
- **THEN** the font size control MAY be hidden or de-emphasized

---

### Requirement: Collapsible AI Results Sidebar

The app SHALL include an `<aside id="ai-panel">` element that displays AI results (OCR text, LLM responses). The panel SHALL be collapsible and MUST NOT block or overlap the annotation canvas workspace when expanded.

#### Scenario: Panel is collapsed by default
- **WHEN** the app loads with no AI results
- **THEN** the AI panel MUST have the `collapsed` CSS class and occupy no canvas workspace

#### Scenario: Expanding the AI panel
- **WHEN** the user clicks the toggle button (`data-action="toggle-ai-panel"`) or triggers an AI action
- **THEN** the AI panel MUST expand, revealing its content area
- **THEN** the canvas container MUST resize to accommodate the panel without overlapping

#### Scenario: Collapsing the AI panel
- **WHEN** the user clicks the toggle button while the panel is expanded
- **THEN** the panel MUST collapse, restoring full canvas workspace

### Requirement: OCR Results Display

The AI panel SHALL contain a `#ocr-results` section that displays extracted text from OCR operations.

#### Scenario: Displaying OCR results
- **WHEN** an OCR operation completes successfully
- **THEN** the `#ocr-results` section MUST become visible
- **THEN** the extracted text MUST be displayed in the section
- **THEN** the AI panel MUST expand if it was collapsed

#### Scenario: No OCR results
- **WHEN** no OCR operation has been performed
- **THEN** the `#ocr-results` section MUST have the `hidden` class

### Requirement: LLM Response Display

The AI panel SHALL contain a `#llm-results` section that displays responses from LLM vision analysis.

#### Scenario: Displaying LLM analysis
- **WHEN** an LLM analysis operation completes successfully
- **THEN** the `#llm-results` section MUST become visible
- **THEN** the LLM response text MUST be rendered in the section
- **THEN** the AI panel MUST expand if it was collapsed

#### Scenario: Processing indicator
- **WHEN** an AI operation (OCR or LLM) is in progress
- **THEN** the AI panel MUST display a spinner icon with the text "Processing..." in the relevant results section (`#ocr-results` or `#llm-results`)

---

### Requirement: Status Bar Layout

The app SHALL render a `<footer id="statusbar">` at the bottom of the window displaying contextual information in discrete spans.

#### Scenario: Status bar elements present on load
- **WHEN** the app window is visible
- **THEN** the status bar MUST contain spans for: dimensions (`#status-dimensions`), zoom level (`#status-zoom`), active tool (`#status-tool`), and status message (`#status-message`)

### Requirement: Dimensions Display

The `#status-dimensions` span SHALL display the current screenshot dimensions in `WxH` format (e.g., "1920x1080").

#### Scenario: Dimensions update on screenshot load
- **WHEN** a screenshot is captured or loaded
- **THEN** `#status-dimensions` MUST update to show the image width and height in pixels

#### Scenario: No screenshot loaded
- **WHEN** no screenshot is loaded
- **THEN** `#status-dimensions` MUST be empty or show a placeholder

### Requirement: Zoom Level Display

The `#status-zoom` span SHALL display the current canvas zoom level as a percentage (e.g., "100%").

#### Scenario: Zoom level reflects canvas state
- **WHEN** the user zooms in or out
- **THEN** `#status-zoom` MUST update to reflect the current zoom percentage

### Requirement: Active Tool Display

The `#status-tool` span SHALL display the name of the currently active tool.

#### Scenario: Tool name updates on tool switch
- **WHEN** the user switches to the rectangle tool
- **THEN** `#status-tool` MUST display "Rectangle"

### Requirement: Status Messages

The `#status-message` span SHALL display transient status messages (e.g., "Saved to ~/Pictures/Fotos/screenshot.png", "Copied to clipboard", "OCR complete").

#### Scenario: Status message shown after save
- **WHEN** a save operation completes
- **THEN** `#status-message` MUST display a confirmation message including the file path

#### Scenario: Status message clears after timeout
- **WHEN** a status message is displayed
- **THEN** it MUST automatically clear after 4 seconds

---

### Requirement: Annotation Tool Shortcuts

The app SHALL support single-key shortcuts for switching annotation tools. The canonical key assignment for each tool is defined in the **annotation-tools** spec (each tool requirement specifies its activation shortcut). The ui-shell is responsible for the keyboard event listener and dispatch; the annotation-tools spec is the source of truth for which key maps to which tool.

The following table summarizes the current mappings (see annotation-tools spec for authoritative definitions):

| Key | Tool |
|-----|------|
| `V` | Select |
| `A` | Arrow |
| `R` | Rectangle |
| `E` | Ellipse |
| `T` | Text |
| `B` | Blur |
| `N` | Step Number |
| `F` | Freehand |
| `H` | Highlight |
| `C` | Crop |

These shortcuts MUST NOT fire when a text input or textarea is focused.

#### Scenario: Pressing a tool shortcut key activates the tool
- **WHEN** the user presses `R` while no text input is focused
- **THEN** the active tool MUST switch to `rect`
- **THEN** the toolbar MUST visually highlight the rectangle button

#### Scenario: Tool shortcuts suppressed during text input
- **WHEN** the user presses `R` while a text input or textarea is focused
- **THEN** the keypress MUST be handled as normal text input and MUST NOT switch tools

### Requirement: Undo and Redo Shortcuts

The app SHALL support `Ctrl+Z` for undo and `Ctrl+Shift+Z` for redo.

#### Scenario: Ctrl+Z performs undo
- **WHEN** the user presses `Ctrl+Z`
- **THEN** the most recent annotation command MUST be undone
- **THEN** the canvas MUST re-render to reflect the undo

#### Scenario: Ctrl+Shift+Z performs redo
- **WHEN** the user presses `Ctrl+Shift+Z`
- **THEN** the most recently undone command MUST be re-applied

### Requirement: Clipboard and Save Shortcuts

The app SHALL support `Ctrl+C` for copy to clipboard, `Ctrl+S` for save, and `Ctrl+Shift+S` for save-as.

#### Scenario: Ctrl+C copies annotated screenshot to clipboard
- **WHEN** the user presses `Ctrl+C` with a screenshot loaded and no text selection active
- **THEN** the composited image (screenshot + annotations) MUST be copied to the system clipboard

#### Scenario: Ctrl+S saves the image
- **WHEN** the user presses `Ctrl+S`
- **THEN** the save workflow MUST be triggered

#### Scenario: Ctrl+Shift+S opens save-as dialog
- **WHEN** the user presses `Ctrl+Shift+S`
- **THEN** a save-as file dialog MUST appear allowing the user to choose path and format

### Requirement: Delete and Escape Shortcuts

The app SHALL support `Delete` to delete the selected annotation and `Escape` to deselect or cancel the current tool action.

#### Scenario: Delete removes selected annotation
- **WHEN** an annotation is selected and the user presses `Delete`
- **THEN** the selected annotation MUST be removed from the canvas
- **THEN** the deletion MUST be recorded in the undo history

#### Scenario: Escape deselects current selection
- **WHEN** the user presses `Escape` while an annotation is selected
- **THEN** the selection MUST be cleared

#### Scenario: Escape cancels in-progress tool action
- **WHEN** the user presses `Escape` while drawing an annotation (e.g., mid-drag)
- **THEN** the in-progress annotation MUST be discarded without committing

### Requirement: Select All Shortcut

The app SHALL support `Ctrl+A` to select all annotations.

#### Scenario: Ctrl+A selects all annotations
- **WHEN** the user presses `Ctrl+A`
- **THEN** all annotations on the canvas MUST become selected

### Requirement: Zoom Shortcuts

The app SHALL support `+` (or `=`) to zoom in, `-` to zoom out, and `Ctrl+0` to reset zoom to 100%.

#### Scenario: Plus key zooms in
- **WHEN** the user presses `+` or `=`
- **THEN** the canvas zoom level MUST increase by one step

#### Scenario: Minus key zooms out
- **WHEN** the user presses `-`
- **THEN** the canvas zoom level MUST decrease by one step

#### Scenario: Ctrl+0 resets zoom
- **WHEN** the user presses `Ctrl+0`
- **THEN** the canvas zoom level MUST reset to 1.0 (100%)

### Requirement: Pan Shortcut

The app SHALL support `Space` + drag for panning the canvas.

#### Scenario: Space-drag pans the canvas
- **WHEN** the user holds `Space` and drags the mouse on the canvas
- **THEN** the canvas MUST pan in the direction of the drag
- **THEN** the cursor MUST change to a grab/hand icon while Space is held

---

### Requirement: System Theme Detection

The app SHALL detect the OS color scheme preference using the `prefers-color-scheme` CSS media query and apply the corresponding light or dark theme by default.

#### Scenario: Dark theme applied on dark OS preference
- **WHEN** the OS is set to dark mode and the theme setting is `"system"`
- **THEN** the dark theme CSS custom properties MUST be active
- **THEN** `--bg-primary` MUST be `#1e1e1e`

#### Scenario: Light theme applied on light OS preference
- **WHEN** the OS is set to light mode and the theme setting is `"system"`
- **THEN** the light theme CSS custom properties MUST be active
- **THEN** `--bg-primary` MUST be `#ffffff`

### Requirement: CSS Custom Properties for Theming

All UI colors SHALL be defined as CSS custom properties on `:root`. The following properties MUST be defined:

- `--bg-primary` -- primary background
- `--bg-secondary` -- secondary background
- `--bg-toolbar` -- toolbar background
- `--text-primary` -- primary text color
- `--text-secondary` -- secondary text color
- `--border` -- border color
- `--accent` -- accent/brand color
- `--accent-hover` -- accent hover state

Light theme defaults: `--bg-primary: #ffffff`, `--bg-secondary: #f5f5f5`, `--bg-toolbar: #e8e8e8`, `--text-primary: #1a1a1a`, `--text-secondary: #666666`, `--border: #d0d0d0`, `--accent: #2563eb`, `--accent-hover: #1d4ed8`.

Dark theme defaults: `--bg-primary: #1e1e1e`, `--bg-secondary: #2d2d2d`, `--bg-toolbar: #333333`, `--text-primary: #e0e0e0`, `--text-secondary: #999999`, `--border: #444444`, `--accent: #3b82f6`, `--accent-hover: #60a5fa`.

#### Scenario: Custom properties applied in light mode
- **WHEN** the light theme is active
- **THEN** all eight CSS custom properties MUST match the light theme defaults

#### Scenario: Custom properties applied in dark mode
- **WHEN** the dark theme is active
- **THEN** all eight CSS custom properties MUST match the dark theme defaults

### Requirement: Theme Setting with Manual Override

The UI settings SHALL include a theme preference with three options: `"system"`, `"light"`, and `"dark"`. The default SHALL be `"system"`.

#### Scenario: Manual dark override
- **WHEN** the user sets the theme preference to `"dark"`
- **THEN** the dark theme MUST be applied regardless of the OS color scheme

#### Scenario: Manual light override
- **WHEN** the user sets the theme preference to `"light"`
- **THEN** the light theme MUST be applied regardless of the OS color scheme

#### Scenario: System option follows OS
- **WHEN** the user sets the theme preference to `"system"` and the OS switches from light to dark
- **THEN** the app theme MUST switch to dark automatically

### Requirement: CSS Architecture Constraints

The app SHALL use no CSS frameworks. Layout SHALL be achieved with CSS Grid. Theming SHALL use CSS custom properties. Only minimal hand-written utility classes are permitted.

#### Scenario: No CSS framework dependencies
- **WHEN** the app is built
- **THEN** no CSS framework files (e.g., Bootstrap, Tailwind, Bulma) SHALL be present in the frontend assets

---

### Requirement: Default Window Dimensions

The main Tauri window SHALL open at 1200x800 pixels with a minimum size of 800x600 pixels.

#### Scenario: Window opens at default size
- **WHEN** the app starts for the first time
- **THEN** the window width MUST be 1200 pixels
- **THEN** the window height MUST be 800 pixels

#### Scenario: Window cannot be resized below minimum
- **WHEN** the user attempts to resize the window below 800x600
- **THEN** the window MUST NOT shrink below 800 pixels wide or 600 pixels tall

### Requirement: Window Properties

The main window SHALL be resizable, have native window decorations, and be initially hidden (shown programmatically after initialization).

#### Scenario: Window is resizable
- **WHEN** the user drags a window edge or corner
- **THEN** the window MUST resize accordingly (within minimum bounds)

#### Scenario: Window has native decorations
- **WHEN** the app window is visible
- **THEN** the window MUST display the platform-native title bar and window controls (minimize, maximize, close)

#### Scenario: Window is initially hidden
- **WHEN** the Tauri app process starts
- **THEN** the window MUST NOT be visible until the frontend signals readiness
- **THEN** this prevents a flash of unstyled content during initialization

---

### Requirement: Preferences UI

The app SHALL provide a settings modal (`settings.js`) accessible from the UI that allows the user to configure preferences across four categories: Capture, Annotation, AI, and UI.

#### Scenario: Opening the settings modal
- **WHEN** the user triggers the settings action (e.g., via menu or toolbar)
- **THEN** a modal dialog MUST appear overlaying the main workspace

#### Scenario: Closing the settings modal
- **WHEN** the user clicks a close button or presses Escape in the settings modal
- **THEN** the modal MUST close and return focus to the main workspace

### Requirement: Settings Modal Content

The settings modal SHALL expose all user preferences defined in the **settings-credentials** spec, organized into the same four sections: Capture, Annotation, AI, and UI. The settings-credentials spec is the single source of truth for key names, types, and default values; this spec defines only the UI presentation.

Each preference key defined in settings-credentials SHALL have a corresponding form control in the appropriate settings section. API key entry fields SHALL invoke the `set_api_key` Tauri command to persist keys to the OS keychain. API keys MUST NOT be stored in config files, localStorage, or any frontend-accessible storage.

#### Scenario: Changing a capture setting
- **WHEN** the user changes the default capture mode to `"fullscreen"` and saves settings
- **THEN** the persisted `capture.defaultMode` setting MUST be `"fullscreen"`

#### Scenario: Setting an API key
- **WHEN** the user enters an Anthropic API key and saves settings
- **THEN** the key MUST be persisted to the OS keychain via the `set_api_key` Tauri command
- **THEN** the key MUST NOT be written to any config file or localStorage

#### Scenario: Hiding the status bar
- **WHEN** the user sets "Show status bar" to false and saves
- **THEN** the status bar MUST be hidden from the UI layout

#### Scenario: Hiding the AI panel
- **WHEN** the user sets "Show AI panel" to false and saves
- **THEN** the AI panel MUST be hidden and the canvas MUST use the full available width

### Requirement: Settings Persistence

All settings MUST be persisted via the Tauri `tauri-plugin-store` backend (`set_settings` / `get_settings` commands). Settings MUST be loaded on app startup and applied before the window becomes visible.

#### Scenario: Settings survive app restart
- **WHEN** the user changes settings, saves, and restarts the app
- **THEN** all changed settings MUST be restored to their saved values on next launch

---

### Requirement: Export Dialog UI

The app SHALL provide an export dialog (`export-dialog.js`) that presents save, copy, and upload options for the annotated screenshot.

#### Scenario: Opening the export dialog
- **WHEN** the user triggers the save-as action (`Ctrl+Shift+S` or the save-as toolbar button)
- **THEN** a custom export dialog MUST appear with three output options: Save to File, Copy to Clipboard, and Export Annotations as JSON

> **Note**: The export dialog is distinct from the quick-save flow (`Ctrl+S`), which saves directly to the default directory without opening a dialog. The "Save to File" option within the export dialog uses the native file-picker dialog via `tauri-plugin-dialog` for path selection.

### Requirement: Save to File

The export dialog SHALL allow saving the composited image (screenshot + annotations) to a file. Supported formats SHALL include PNG, JPEG, and WebP.

#### Scenario: Saving as PNG
- **WHEN** the user selects PNG format and a destination path in the export dialog
- **THEN** the composited image MUST be saved as a PNG file at the chosen path
- **THEN** a success status message MUST appear in the status bar

#### Scenario: Saving as JPEG with quality setting
- **WHEN** the user selects JPEG format
- **THEN** the export dialog MUST allow setting the JPEG quality (1-100)
- **THEN** the image MUST be saved at the specified quality

### Requirement: Copy to Clipboard

The export dialog SHALL include an option to copy the composited image to the system clipboard.

#### Scenario: Copy to clipboard from export dialog
- **WHEN** the user clicks the copy-to-clipboard option
- **THEN** the composited image MUST be placed on the system clipboard
- **THEN** a confirmation status message MUST appear in the status bar

### Requirement: Export Annotations as JSON

The export dialog SHALL include an option to export annotations as a JSON file for later reimport.

#### Scenario: Exporting annotations
- **WHEN** the user selects the export-annotations option
- **THEN** the current annotation array MUST be serialized to JSON
- **THEN** the user MUST be prompted to choose a save location for the JSON file

---

### Requirement: Error Notification System

The app SHALL display user-facing error notifications as toast messages in a `#toast-container` element positioned at the bottom-right of the viewport. Each toast SHALL have a severity level (`error`, `warning`, `info`) indicated by color (red, yellow, blue respectively). Toasts SHALL auto-dismiss after 6 seconds. The user SHALL be able to dismiss a toast early by clicking a close button on it. A maximum of 3 toasts SHALL be visible simultaneously; additional toasts SHALL queue and appear as earlier ones dismiss.

#### Scenario: Display error toast
- **WHEN** an operation fails (e.g., save failure, API error, clipboard error)
- **THEN** a toast with severity `error` SHALL appear in `#toast-container` with a descriptive message
- **THEN** the toast SHALL have a red accent color and a close button

#### Scenario: Toast auto-dismiss
- **WHEN** a toast is displayed
- **THEN** it SHALL automatically fade out and be removed after 6 seconds

#### Scenario: Toast queue when at capacity
- **WHEN** 3 toasts are visible and a new error occurs
- **THEN** the new toast SHALL be queued and displayed when one of the visible toasts is dismissed

#### Scenario: Manual dismiss
- **WHEN** the user clicks the close button on a toast
- **THEN** the toast SHALL be immediately removed and any queued toast SHALL appear

---

### Requirement: Settings Modal Trigger

The settings modal SHALL be accessible via a gear icon button (`data-action="open-settings"`) in the toolbar's output tools group, and via the keyboard shortcut `Ctrl+,` (comma). The gear button SHALL be placed after the Save As button in the output tools group.

#### Scenario: Open settings via toolbar button
- **WHEN** the user clicks the gear icon button in the toolbar
- **THEN** the settings modal MUST appear overlaying the main workspace

#### Scenario: Open settings via keyboard shortcut
- **WHEN** the user presses `Ctrl+,`
- **THEN** the settings modal MUST appear overlaying the main workspace

---

### Requirement: Basic Accessibility

The app SHALL meet baseline accessibility requirements for keyboard and screen reader users:

- All toolbar buttons MUST have `aria-label` attributes describing their function (e.g., `aria-label="Arrow tool (A)"`)
- Active tool buttons MUST have `aria-pressed="true"`
- The settings modal MUST trap focus while open (Tab cycles within the modal)
- The settings modal MUST have `role="dialog"` and `aria-modal="true"`
- All interactive elements MUST be reachable via Tab key navigation
- Focus order MUST follow the visual layout (left-to-right, top-to-bottom)

#### Scenario: Toolbar buttons have accessible labels
- **WHEN** a screen reader user navigates to a toolbar button
- **THEN** the screen reader MUST announce the tool name and keyboard shortcut from the `aria-label`

#### Scenario: Modal traps focus
- **WHEN** the settings modal is open and the user presses Tab
- **THEN** focus MUST cycle through the modal's interactive elements without escaping to the main page

#### Scenario: Keyboard-only tool switching
- **WHEN** a keyboard-only user presses Tab to navigate the toolbar and presses Enter on a tool button
- **THEN** the corresponding tool MUST become active
