/**
 * canvas-runtime.js
 *
 * Executes drawing scripts on a double-buffered HTML5 canvas.
 *
 * Architecture:
 *   - All drawing happens on an off-screen `bufferCanvas` first.
 *   - The result is composited onto the visible `drawCanvas` only when a
 *     frame is ready (end of a non-animation script, or an explicit frame
 *     flush during animation).
 *   - Coordinates passed to drawing commands are in logical canvas units and
 *     are converted to physical pixels by `toPx` (from canvas-utils).
 */

import { CANVAS_W, CANVAS_H, deg2rad, toPx, tokenize, randomColorString } from './canvas-utils.js';

const logEl      = document.getElementById('logs');
const drawCanvas = document.getElementById('canvas');
const drawCtx    = drawCanvas.getContext('2d', { alpha: false });

// Off-screen buffer – all drawing targets this canvas to avoid partial-frame flicker.
const bufferCanvas        = document.createElement('canvas');
bufferCanvas.width        = 1000;
bufferCanvas.height       = 1000;
const ctx                 = bufferCanvas.getContext('2d', { alpha: false });

let isAnimation = false;
let isCancelled = false;

// Cap DPR at 3 to avoid excessive memory usage on very high-density displays.
const DPR = Math.max(1, Math.min(3, window.devicePixelRatio || 1));

/** Palette of lazily-generated random colors, indexed by `RandomColor<N>` tokens. */
let randomColors = [];

/** Safari requires a special repaint workaround after compositing (see `drawScript`). */
let isSafari = false;

// Size both canvases to physical pixels and apply a DPR scale transform so
// all subsequent draw calls can use logical (CSS) pixel coordinates.
drawCanvas.width    = Math.floor(CANVAS_W * DPR);
drawCanvas.height   = Math.floor(CANVAS_H * DPR);
bufferCanvas.width  = drawCanvas.width;
bufferCanvas.height = drawCanvas.height;
ctx.setTransform(DPR, 0, 0, DPR, 0, 0);

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Write a message to the on-page log element.
 * Non-string values are coerced with `String()`.
 *
 * @param {*} text - Message to display.
 */
export function log(text) {
  if (typeof text !== 'string') text = String(text);
  logEl.textContent = text;
}

/**
 * Reset runtime state before executing a new script.
 * Clears the random-color palette and wipes the canvas.
 */
export function setup() {
  randomColors = [];
  clearCanvas();
}

/**
 * Return whether the current execution has been cancelled.
 *
 * @returns {boolean}
 */
export function checkIsCancelled() {
  return isCancelled;
}

/**
 * Cancel (or un-cancel) the current execution.
 * Also resets the animation flag so the render loop stops.
 *
 * @param {boolean} [value=true]
 */
export function cancelNow(value = true) {
  isCancelled = value;
  isAnimation = false;
}

/**
 * Tell the runtime whether it is running inside Safari.
 *
 * @param {boolean} value
 */
export function setIsSafari(value) {
  isSafari = value;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/**
 * Fill the buffer canvas with a solid background color.
 * Resets the transform temporarily so the clear covers the full physical
 * canvas regardless of the DPR scale transform.
 *
 * @param {string} [color='#0a0f1f']
 */
function clearCanvas(color = '#0a0f1f') {
  ctx.save(); ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, drawCanvas.width, drawCanvas.height);
  ctx.restore();

  ctx.save(); ctx.fillStyle = color;
  ctx.fillRect(0, 0, CANVAS_W, CANVAS_H);
  ctx.restore();
}

/**
 * Apply stroke/fill/lineWidth options from a parsed options object to `ctx`.
 *
 * @param {{ width?: number, stroke?: string, fill?: string }} opts
 */
function applyStyle(opts) {
  ctx.lineWidth = opts.width ?? 1;
  if (opts.stroke) ctx.strokeStyle = opts.stroke;
  if (opts.fill)   ctx.fillStyle   = opts.fill;
}

/**
 * Parse `key=value` pairs from a token array into a drawing-options object.
 *
 * Supported keys: `width`, `stroke`, `fill`, `ccw`.
 *
 * `RandomColor` values are resolved against the shared `randomColors` palette:
 * - `RandomColor`  → appends a new random color and uses it.
 * - `RandomColor0`, `RandomColor1`, … → uses (or lazily creates) the color at
 *   that index, so the same index always returns the same color within a run.
 *
 * @param {string[]} tokens   - Full token array for the current script line.
 * @param {number}   startIdx - Index of the first key=value token.
 * @returns {{ width?: number, stroke?: string, fill?: string, ccw?: boolean }}
 */
function parseOptions(tokens, startIdx) {
  const o = {};
  for (let i = startIdx; i < tokens.length; i++) {
    const t  = tokens[i];
    const eq = t.indexOf('=');
    if (eq > 0) {
      let k = t.slice(0, eq);
      let v = t.slice(eq + 1);

      if (v.startsWith('RandomColor')) {
        let idx = parseInt(v.slice(11));
        if (isNaN(idx)) {
          randomColors.push(randomColorString());
          idx = randomColors.length - 1;
        }
        while (idx >= randomColors.length) {
          randomColors.push(randomColorString());
        }
        v = randomColors[idx];
      }

      if      (k === 'width')  o.width  = Number(v);
      else if (k === 'stroke') o.stroke = v;
      else if (k === 'fill')   o.fill   = v;
      else if (k === 'ccw')    o.ccw    = /^(1|true|yes)$/i.test(v);
    }
  }
  return o;
}

/**
 * Draw a full circle.
 *
 * @param {number|string} cx - Centre X in logical units.
 * @param {number|string} cy - Centre Y in logical units.
 * @param {number|string} r  - Radius in logical units.
 * @param {object}        o  - Style options from `parseOptions`.
 */
function drawCircle(cx, cy, r, o) {
  ctx.beginPath();
  ctx.arc(toPx(cx, 'x'), toPx(cy, 'y'), toPx(r, 'x'), 0, Math.PI * 2);
  if (o.fill)            ctx.fill();
  if (o.stroke || !o.fill) ctx.stroke();
}

/**
 * Draw an axis-aligned rectangle defined by two corner points.
 *
 * @param {number|string} x - Left edge in logical units.
 * @param {number|string} y - Top edge in logical units.
 * @param {number|string} w - Right edge in logical units.
 * @param {number|string} h - Bottom edge in logical units.
 * @param {object}        o - Style options from `parseOptions`.
 */
function drawRect(x, y, w, h, o) {
  const X = toPx(x, 'x'), Y = toPx(y, 'y'), W = toPx(w, 'x'), H = toPx(h, 'y');
  if (o.fill)            ctx.fillRect  (X, Y, W - X, H - Y);
  if (o.stroke || !o.fill) ctx.strokeRect(X, Y, W - X, H - Y);
}

/**
 * Draw a straight line segment.
 *
 * @param {number|string} x1 - Start X in logical units.
 * @param {number|string} y1 - Start Y in logical units.
 * @param {number|string} x2 - End X in logical units.
 * @param {number|string} y2 - End Y in logical units.
 * @param {object}        o  - Style options from `parseOptions`.
 */
function drawLine(x1, y1, x2, y2, o) {
  ctx.beginPath();
  ctx.moveTo(toPx(x1, 'x'), toPx(y1, 'y'));
  ctx.lineTo(toPx(x2, 'x'), toPx(y2, 'y'));
  ctx.stroke();
}

/**
 * Draw a closed polygon from a flat array of alternating X/Y coordinates.
 * Requires at least two points (four numbers).
 *
 * @param {number[]} nums - Flat coordinate list: [x0, y0, x1, y1, …].
 * @param {object}   o    - Style options from `parseOptions`.
 */
function drawPolygon(nums, o) {
  if (nums.length < 4) return;
  ctx.beginPath();
  ctx.moveTo(toPx(nums[0], 'x'), toPx(nums[1], 'y'));
  for (let i = 2; i < nums.length; i += 2) {
    ctx.lineTo(toPx(nums[i], 'x'), toPx(nums[i + 1], 'y'));
  }
  ctx.closePath();
  if (o.fill)            ctx.fill();
  if (o.stroke || !o.fill) ctx.stroke();
}

/**
 * Draw a circular arc.
 *
 * @param {number|string} cx  - Centre X in logical units.
 * @param {number|string} cy  - Centre Y in logical units.
 * @param {number|string} r   - Radius in logical units.
 * @param {number}        a0  - Start angle in degrees.
 * @param {number}        a1  - End angle in degrees.
 * @param {boolean}       ccw - Draw counter-clockwise when true.
 * @param {object}        o   - Style options from `parseOptions`.
 */
function drawArc(cx, cy, r, a0, a1, ccw, o) {
  ctx.beginPath();
  ctx.arc(toPx(cx, 'x'), toPx(cy, 'y'), toPx(r, 'x'), deg2rad(a0), deg2rad(a1), !!ccw);
  if (o.fill)            ctx.fill();
  if (o.stroke || !o.fill) ctx.stroke();
}

// ---------------------------------------------------------------------------
// Script executor
// ---------------------------------------------------------------------------

/**
 * Interpret and render an array of drawing-script lines onto the buffer canvas,
 * then composite to the visible canvas when appropriate.
 *
 * Supported commands (case-insensitive):
 *   - `circle <cx> <cy> <r> [opts]`
 *   - `rectangle <x> <y> <w> <h> [opts]`
 *   - `line <x1> <y1> <x2> <y2> [opts]`
 *   - `polygon <x0> <y0> … [opts]`
 *   - `arc <cx> <cy> <r> <a0> <a1> [opts]`
 *   - `bg|background [color]`
 *   - `animate`
 *   - `clear`
 *   - `error <msg>`
 *   - `print <msg>`
 *
 * The `animate` command sets a flag that defers compositing until
 * `should_draw_frame` is explicitly true, enabling frame-by-frame animation.
 *
 * @param {string[]} script            - Lines of the drawing script to execute.
 * @param {boolean}  [should_draw_frame=false] - Force a composite to the visible
 *                                               canvas even when in animation mode.
 */
export function drawScript(script, should_draw_frame = false) {
  ctx.save(); ctx.lineJoin = 'round'; ctx.lineCap = 'round';

  for (var i = 0; i < script.length; i += 1) {
    let raw = script[i];
    if (isCancelled) { return; }

    const line = raw.trim();
    if (!line || line.startsWith('//')) continue;

    const tok = tokenize(line);
    if (!tok.length) continue;

    const cmd = tok[0].toLowerCase();
    try {
      switch (cmd) {
        case 'circle': {
          const [_, cx, cy, r] = tok;
          const o = parseOptions(tok, 4);
          applyStyle(o); drawCircle(cx, cy, r, o);
          break;
        }
        case 'rectangle': {
          const [_, x, y, w, h] = tok;
          const o = parseOptions(tok, 5);
          applyStyle(o); drawRect(x, y, w, h, o);
          break;
        }
        case 'line': {
          const [_, x1, y1, x2, y2] = tok;
          const o = parseOptions(tok, 5);
          applyStyle(o); drawLine(x1, y1, x2, y2, o);
          break;
        }
        case 'polygon': {
          const nums = []; let i = 1;
          for (; i < tok.length; i++) {
            if (tok[i].includes('=')) break;
            nums.push(Number(tok[i]));
          }
          const o = parseOptions(tok, i);
          applyStyle(o); drawPolygon(nums, o);
          break;
        }
        case 'arc': {
          const [_, cx, cy, r, a0, a1] = tok;
          const o = parseOptions(tok, 6);
          applyStyle(o); drawArc(cx, cy, r, Number(a0), Number(a1), !!o.ccw, o);
          break;
        }
        case 'bg':
        case 'background': {
          const color = tok[1] || '#0a0f1f';
          clearCanvas(color);
          break;
        }
        case 'animate': { isAnimation = true; break; }
        case 'clear':   { clearCanvas(); break; }
        case 'error':   { alert('Error: ' + raw); break; }
        case 'print': {
          const msg = raw.slice(5).trim();
          console.log('Print:' + msg);
          break;
        }
        default: /* ignore unknown commands */ break;
      }
    } catch (e) { console.warn('Error:', line, e); }
  }

  ctx.restore();

  // Composite the buffer onto the visible canvas.
  // In animation mode, only do this when explicitly requested (i.e. per-frame).
  if (!isAnimation || should_draw_frame) {
    drawCtx.clearRect(0, 0, drawCanvas.width, drawCanvas.height);
    drawCtx.drawImage(bufferCanvas, 0, 0);
    if (isSafari) {
      drawCanvas.getContext('2d').getImageData(0, 0, 1, 1); // Force repaint
    }
  }
}

// ---------------------------------------------------------------------------
// Resize handling
// ---------------------------------------------------------------------------

/**
 * Synchronise the internal canvas resolutions to the element's current
 * CSS size × device pixel ratio.
 *
 * Should be called on window `resize` events so the buffer and visible canvas
 * stay in sync with layout changes.
 */
export function resizeCanvases() {
  const rect = drawCanvas.getBoundingClientRect();
  const dpr  = window.devicePixelRatio || 1;

  // Match canvas internal size to actual visible size * device pixel ratio.
  const width  = Math.floor(rect.width  * dpr);
  const height = Math.floor(rect.height * dpr);

  if (drawCanvas.width !== width || drawCanvas.height !== height) {
    drawCanvas.width    = width;
    drawCanvas.height   = height;
    bufferCanvas.width  = width;
    bufferCanvas.height = height;
  }
}
