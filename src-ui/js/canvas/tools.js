/// Tool definitions and registry.
/// Maps tool names to their handler modules.

// TODO: import individual tool modules when implemented
// import { ArrowTool } from './tool-arrow.js';
// etc.

export const TOOL_SHORTCUTS = {
  'v': 'select',
  'a': 'arrow',
  'r': 'rect',
  'e': 'ellipse',
  't': 'text',
  'b': 'blur',
  'n': 'step',
  'f': 'freehand',
  'h': 'highlight',
  'c': 'crop',
};

export function getToolHandler(toolName) {
  // TODO: return tool handler instance
  return null;
}
