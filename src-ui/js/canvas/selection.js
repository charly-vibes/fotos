/// Selection tool — select, move, resize, and delete annotations.

import { pointInRect, pointToSegmentDist, pointInEllipse } from '../utils/geometry.js';

// 8 handle descriptors for bbox-based shapes, in NW..SE order.
const HANDLE_DEFS = [
  { id: 'nw', cursor: 'nw-resize', getPos: b => ({ x: b.x,           y: b.y           }) },
  { id: 'n',  cursor: 'n-resize',  getPos: b => ({ x: b.x + b.w / 2, y: b.y           }) },
  { id: 'ne', cursor: 'ne-resize', getPos: b => ({ x: b.x + b.w,     y: b.y           }) },
  { id: 'w',  cursor: 'w-resize',  getPos: b => ({ x: b.x,           y: b.y + b.h / 2 }) },
  { id: 'e',  cursor: 'e-resize',  getPos: b => ({ x: b.x + b.w,     y: b.y + b.h / 2 }) },
  { id: 'sw', cursor: 'sw-resize', getPos: b => ({ x: b.x,           y: b.y + b.h     }) },
  { id: 's',  cursor: 's-resize',  getPos: b => ({ x: b.x + b.w / 2, y: b.y + b.h     }) },
  { id: 'se', cursor: 'se-resize', getPos: b => ({ x: b.x + b.w,     y: b.y + b.h     }) },
];

// Pixel tolerance for arrow/freehand hit testing (image-space pixels).
const HIT_TOLERANCE = 6;

export class SelectionManager {
  #selectedList = []; // ordered array of selected annotation objects

  // ── Selection state ───────────────────────────────────────────────────────

  // Set a single selection (normal click).
  select(annotation) { this.#selectedList = [annotation]; }

  // Clear all selections.
  deselect() { this.#selectedList = []; }

  // Toggle an annotation in/out of the selection (Ctrl+click).
  toggle(annotation) {
    const idx = this.#selectedList.findIndex(a => a.id === annotation.id);
    if (idx >= 0) {
      this.#selectedList = this.#selectedList.filter((_, i) => i !== idx);
    } else {
      this.#selectedList = [...this.#selectedList, annotation];
    }
  }

  // Range-select from the first selected annotation to target by z-order (Shift+click).
  selectRange(annotations, target) {
    const anchor = this.#selectedList[0];
    if (!anchor) { this.#selectedList = [target]; return; }
    const anchorIdx = annotations.findIndex(a => a.id === anchor.id);
    const targetIdx = annotations.findIndex(a => a.id === target.id);
    if (anchorIdx < 0 || targetIdx < 0) { this.#selectedList = [target]; return; }
    const lo = Math.min(anchorIdx, targetIdx);
    const hi = Math.max(anchorIdx, targetIdx);
    this.#selectedList = annotations.slice(lo, hi + 1).filter(a => !a.locked);
  }

  // Select all non-locked annotations (Ctrl+A).
  selectAll(annotations) {
    this.#selectedList = annotations.filter(a => !a.locked);
  }

  // Primary selected annotation (first in list); null when nothing selected.
  get selected() { return this.#selectedList[0] ?? null; }

  // All selected annotations as an array.
  get selectedList() { return this.#selectedList; }

  // Number of selected annotations.
  get count() { return this.#selectedList.length; }

  // Returns true if an annotation with the given id is selected.
  isSelected(id) { return this.#selectedList.some(a => a.id === id); }

  // Update one annotation reference in the selection (e.g. after undo/redo replaces objects).
  updateSelected(annotation) {
    this.#selectedList = this.#selectedList.map(a => a.id === annotation.id ? annotation : a);
  }

  // Sync selected list with a fresh annotations array (drop stale references).
  syncWithAnnotations(annotations) {
    this.#selectedList = this.#selectedList
      .map(sel => annotations.find(a => a.id === sel.id))
      .filter(Boolean);
  }

  // ── Hit testing ──────────────────────────────────────────────────────────────

  // Hit-test excluding locked annotations (used for selection click).
  hitTest(x, y, annotations) {
    for (let i = annotations.length - 1; i >= 0; i--) {
      if (!annotations[i].locked && this.#hitTestOne(x, y, annotations[i])) {
        return { annotation: annotations[i], index: i };
      }
    }
    return null;
  }

  // Hit-test including locked annotations (used for context menu right-click).
  hitTestAll(x, y, annotations) {
    for (let i = annotations.length - 1; i >= 0; i--) {
      if (this.#hitTestOne(x, y, annotations[i])) {
        return { annotation: annotations[i], index: i };
      }
    }
    return null;
  }

  // Returns true if the point is inside any currently selected annotation.
  hitTestSelected(x, y) {
    return this.#selectedList.some(a => this.#hitTestOne(x, y, a));
  }

  #hitTestOne(x, y, anno) {
    switch (anno.type) {
      case 'rect':
      case 'blur':
      case 'highlight':
        return pointInRect(x, y, anno);

      case 'ellipse': {
        const cx = anno.x + anno.width / 2;
        const cy = anno.y + anno.height / 2;
        return pointInEllipse(x, y, cx, cy, Math.abs(anno.width / 2), Math.abs(anno.height / 2));
      }

      case 'arrow': {
        const pts = anno.points;
        if (!pts || pts.length < 2) return false;
        return pointToSegmentDist(x, y, pts[0].x, pts[0].y, pts[1].x, pts[1].y) <= HIT_TOLERANCE;
      }

      case 'freehand': {
        const pts = anno.points;
        if (!pts || pts.length < 2) return false;
        for (let i = 0; i < pts.length - 1; i++) {
          if (pointToSegmentDist(x, y, pts[i].x, pts[i].y, pts[i + 1].x, pts[i + 1].y) <= HIT_TOLERANCE) {
            return true;
          }
        }
        return false;
      }

      case 'text': {
        const size = anno.fontSize || 20;
        const approxW = size * (anno.text?.length || 1) * 0.6;
        return pointInRect(x, y, { x: anno.x, y: anno.y, width: approxW, height: size * 1.4 });
      }

      case 'step': {
        const r = (anno.fontSize || 24) / 2;
        return Math.hypot(x - anno.x, y - anno.y) <= r + HIT_TOLERANCE;
      }

      default:
        return false;
    }
  }

  // ── Bounding box ─────────────────────────────────────────────────────────────

  getBBox(anno) {
    switch (anno.type) {
      case 'rect':
      case 'ellipse':
      case 'blur':
      case 'highlight':
        return { x: anno.x, y: anno.y, w: anno.width || 0, h: anno.height || 0 };

      case 'arrow':
      case 'freehand': {
        const pts = anno.points;
        if (!pts || pts.length === 0) return null;
        const xs = pts.map(p => p.x), ys = pts.map(p => p.y);
        return {
          x: Math.min(...xs), y: Math.min(...ys),
          w: Math.max(...xs) - Math.min(...xs),
          h: Math.max(...ys) - Math.min(...ys),
        };
      }

      case 'text': {
        const size = anno.fontSize || 20;
        const approxW = size * (anno.text?.length || 1) * 0.6;
        return { x: anno.x, y: anno.y, w: approxW, h: size * 1.4 };
      }

      case 'step': {
        const r = (anno.fontSize || 24) / 2;
        return { x: anno.x - r, y: anno.y - r, w: r * 2, h: r * 2 };
      }

      default:
        return null;
    }
  }

  // Combined bounding box of an arbitrary list of annotations.
  getCombinedBBox(list) {
    if (!list || list.length === 0) return null;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const anno of list) {
      const b = this.getBBox(anno);
      if (!b) continue;
      if (b.x < minX) minX = b.x;
      if (b.y < minY) minY = b.y;
      if (b.x + b.w > maxX) maxX = b.x + b.w;
      if (b.y + b.h > maxY) maxY = b.y + b.h;
    }
    if (!isFinite(minX)) return null;
    return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
  }

  // Combined bounding box of the current selection.
  getSelectionBBox() {
    return this.getCombinedBBox(this.#selectedList);
  }

  // ── Handles ──────────────────────────────────────────────────────────────────

  // Returns handles for a single annotation.
  // Arrow → 2 endpoint handles; all others → 8 corner/midpoint handles.
  getHandles(anno) {
    if (anno.type === 'arrow') {
      const pts = anno.points;
      if (!pts || pts.length < 2) return [];
      return [
        { id: 'p0', x: pts[0].x, y: pts[0].y, cursor: 'crosshair' },
        { id: 'p1', x: pts[1].x, y: pts[1].y, cursor: 'crosshair' },
      ];
    }
    const bbox = this.getBBox(anno);
    if (!bbox) return [];
    return HANDLE_DEFS.map(def => ({ id: def.id, cursor: def.cursor, ...def.getPos(bbox) }));
  }

  // Returns 8 corner/midpoint handles for an explicit bounding box.
  getHandlesForBBox(bbox) {
    if (!bbox) return [];
    return HANDLE_DEFS.map(def => ({ id: def.id, cursor: def.cursor, ...def.getPos(bbox) }));
  }

  // Hit-test handles for a single annotation.
  hitTestHandle(x, y, anno, handleRadius) {
    for (const h of this.getHandles(anno)) {
      if (Math.hypot(x - h.x, y - h.y) <= handleRadius) return h;
    }
    return null;
  }

  // Hit-test handles on the combined selection bounding box (multi-select).
  hitTestSelectionHandle(x, y, handleRadius) {
    const bbox = this.getSelectionBBox();
    if (!bbox) return null;
    for (const h of this.getHandlesForBBox(bbox)) {
      if (Math.hypot(x - h.x, y - h.y) <= handleRadius) return h;
    }
    return null;
  }

  // ── Transform helpers ────────────────────────────────────────────────────────

  // Apply a move delta (image-space) to a single orig annotation. Returns a new object.
  applyMove(orig, dx, dy) {
    if (orig.type === 'arrow' || orig.type === 'freehand') {
      return {
        ...orig,
        x: orig.x + dx,
        y: orig.y + dy,
        points: orig.points.map(p => ({ x: p.x + dx, y: p.y + dy })),
      };
    }
    // step stores centre in x/y; text stores top-left — both use x/y directly
    return { ...orig, x: orig.x + dx, y: orig.y + dy };
  }

  // Apply move delta to all annotations in a list. Returns a new array.
  applyMoveAll(origList, dx, dy) {
    return origList.map(orig => this.applyMove(orig, dx, dy));
  }

  // Apply a resize-handle drag to a single annotation.
  // totalDx/totalDy are total delta from drag start. Returns a new annotation object.
  applyResize(orig, handleId, totalDx, totalDy) {
    // Arrow: move individual endpoint
    if (orig.type === 'arrow') {
      const pts = orig.points.map(p => ({ ...p }));
      if (handleId === 'p0') { pts[0].x += totalDx; pts[0].y += totalDy; }
      else if (handleId === 'p1') { pts[1].x += totalDx; pts[1].y += totalDy; }
      const xs = pts.map(p => p.x), ys = pts.map(p => p.y);
      return { ...orig, points: pts, x: Math.min(...xs), y: Math.min(...ys) };
    }

    // text / step: any handle drag = move
    if (orig.type === 'text' || orig.type === 'step') {
      return this.applyMove(orig, totalDx, totalDy);
    }

    // rect / ellipse / blur / highlight / freehand: update bounding box
    return this.#resizeBBox(orig, handleId, totalDx, totalDy);
  }

  // Apply proportional resize to all annotations relative to the original combined bbox.
  applyResizeAll(origList, origCombinedBBox, handleId, totalDx, totalDy) {
    const ob = origCombinedBBox;
    let nx = ob.x, ny = ob.y, nw = ob.w, nh = ob.h;
    switch (handleId) {
      case 'nw': nx += totalDx; ny += totalDy; nw -= totalDx; nh -= totalDy; break;
      case 'n':               ny += totalDy;               nh -= totalDy; break;
      case 'ne':              ny += totalDy; nw += totalDx; nh -= totalDy; break;
      case 'w':  nx += totalDx;             nw -= totalDx;                break;
      case 'e':                             nw += totalDx;                break;
      case 'sw': nx += totalDx;             nw -= totalDx; nh += totalDy; break;
      case 's':                                            nh += totalDy; break;
      case 'se':                            nw += totalDx; nh += totalDy; break;
    }
    nw = Math.max(0, nw);
    nh = Math.max(0, nh);

    if (ob.w === 0 || ob.h === 0) return origList;

    const scaleX = nw / ob.w;
    const scaleY = nh / ob.h;
    return origList.map(orig => this.#scaleAnnotation(orig, ob, nx, ny, scaleX, scaleY));
  }

  #resizeBBox(orig, handleId, totalDx, totalDy) {
    let x = orig.x, y = orig.y, w = orig.width || 0, h = orig.height || 0;

    switch (handleId) {
      case 'nw': x += totalDx; y += totalDy; w -= totalDx; h -= totalDy; break;
      case 'n':               y += totalDy;               h -= totalDy; break;
      case 'ne':              y += totalDy; w += totalDx;  h -= totalDy; break;
      case 'w':  x += totalDx;             w -= totalDx;                break;
      case 'e':                            w += totalDx;                break;
      case 'sw': x += totalDx;             w -= totalDx; h += totalDy;  break;
      case 's':                                          h += totalDy;  break;
      case 'se':                           w += totalDx; h += totalDy;  break;
    }

    w = Math.max(0, w);
    h = Math.max(0, h);

    if (orig.type === 'freehand') {
      const oldW = orig.width || 0, oldH = orig.height || 0;
      if (oldW > 0 && oldH > 0) {
        const scaleX = w / oldW, scaleY = h / oldH;
        const newPoints = orig.points.map(p => ({
          x: x + (p.x - orig.x) * scaleX,
          y: y + (p.y - orig.y) * scaleY,
        }));
        return { ...orig, x, y, width: w, height: h, points: newPoints };
      }
    }

    return { ...orig, x, y, width: w, height: h };
  }

  // Scale a single annotation proportionally within a new bounding box.
  #scaleAnnotation(orig, origBBox, newX, newY, scaleX, scaleY) {
    if (orig.type === 'arrow' || orig.type === 'freehand') {
      const pts = orig.points.map(p => ({
        x: newX + (p.x - origBBox.x) * scaleX,
        y: newY + (p.y - origBBox.y) * scaleY,
      }));
      const xs = pts.map(p => p.x), ys = pts.map(p => p.y);
      return { ...orig, x: Math.min(...xs), y: Math.min(...ys), points: pts };
    }
    if (orig.type === 'step' || orig.type === 'text') {
      // Preserve size; only reposition the anchor point.
      return {
        ...orig,
        x: newX + (orig.x - origBBox.x) * scaleX,
        y: newY + (orig.y - origBBox.y) * scaleY,
      };
    }
    // rect, ellipse, blur, highlight
    return {
      ...orig,
      x: newX + (orig.x - origBBox.x) * scaleX,
      y: newY + (orig.y - origBBox.y) * scaleY,
      width:  (orig.width  || 0) * scaleX,
      height: (orig.height || 0) * scaleY,
    };
  }
}
