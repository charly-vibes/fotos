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
  #selected = null;

  select(annotation) { this.#selected = annotation; }
  deselect()         { this.#selected = null; }
  get selected()     { return this.#selected; }

  // Call after the annotation object is replaced in the store (e.g. after undo/redo).
  updateSelected(annotation) { this.#selected = annotation; }

  // ── Hit testing ──────────────────────────────────────────────────────────────

  hitTest(x, y, annotations) {
    for (let i = annotations.length - 1; i >= 0; i--) {
      if (this.#hitTestOne(x, y, annotations[i])) {
        return { annotation: annotations[i], index: i };
      }
    }
    return null;
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

  // ── Handles ──────────────────────────────────────────────────────────────────

  // Returns [{id, x, y, cursor}, ...] in image coords.
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

  // Hit-test handles. handleRadius is in image pixels (pass zoom-adjusted value).
  // Returns the matching handle object or null.
  hitTestHandle(x, y, anno, handleRadius) {
    for (const h of this.getHandles(anno)) {
      if (Math.hypot(x - h.x, y - h.y) <= handleRadius) return h;
    }
    return null;
  }

  // ── Transform helpers ────────────────────────────────────────────────────────

  // Apply a move delta (image-space) to orig annotation.  Returns a new object.
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

  // Apply a resize-handle drag. totalDx/totalDy are total delta from drag start.
  // Returns a new annotation object.
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

    // rect / ellipse / blur / highlight: update x/y/width/height
    if (orig.type === 'freehand') {
      return this.#resizeBBox(orig, handleId, totalDx, totalDy);
    }
    return this.#resizeBBox(orig, handleId, totalDx, totalDy);
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
}
