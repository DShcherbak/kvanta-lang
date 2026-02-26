// Pure utility functions extracted from canvas-runtime.js.
// No DOM dependencies — safe to import in unit tests.

/** Default canvas width in pixels. */
export const CANVAS_W = 1000;

/** Default canvas height in pixels. */
export const CANVAS_H = 1000;

/**
 * Converts degrees to radians.
 * @param {number} d - Angle in degrees.
 * @returns {number} Angle in radians.
 */
export const deg2rad = d => (d * Math.PI) / 180;

/**
 * Converts a value to pixels. If the value is a percentage string (e.g. `"50%"`),
 * it is resolved relative to the canvas width or height depending on `axis`.
 * Otherwise the value is coerced to a number and returned as-is.
 *
 * @param {number|string} val - A numeric pixel value or a percentage string.
 * @param {'x'|'y'} axis - Which axis to resolve percentages against.
 * @returns {number} The resolved pixel value.
 */
export function toPx(val, axis) {
  if (typeof val === 'string' && val.endsWith('%')) {
    const p = parseFloat(val) / 100;
    return (axis === 'x' ? CANVAS_W : CANVAS_H) * p;
  }
  return +val;
}

/**
 * Splits a source line into an array of whitespace-delimited tokens,
 * discarding leading/trailing whitespace and empty segments.
 *
 * @param {string} line - A single line of source text.
 * @returns {string[]} Array of non-empty tokens.
 */
export function tokenize(line) {
  return line.trim().split(/\s+/).filter(Boolean);
}

/**
 * Generates a random opaque CSS color string in `rgb(r, g, b)` format,
 * with each channel uniformly sampled from 0–254.
 *
 * @returns {string} A CSS color string, e.g. `"rgb(123,45,200)"`.
 */
export function randomColorString() {
  const r = Math.floor(255 * Math.random());
  const g = Math.floor(255 * Math.random());
  const b = Math.floor(255 * Math.random());
  return `rgb(${r},${g},${b})`;
}
