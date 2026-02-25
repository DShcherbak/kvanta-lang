// Pure utility functions extracted from canvas-runtime.js.
// No DOM dependencies — safe to import in unit tests.

export const CANVAS_W = 1000;
export const CANVAS_H = 1000;

export const deg2rad = d => (d * Math.PI) / 180;

export function toPx(val, axis) {
  if (typeof val === 'string' && val.endsWith('%')) {
    const p = parseFloat(val) / 100;
    return (axis === 'x' ? CANVAS_W : CANVAS_H) * p;
  }
  return +val;
}

export function tokenize(line) {
  return line.trim().split(/\s+/).filter(Boolean);
}

export function randomColorString() {
  const r = Math.floor(255 * Math.random());
  const g = Math.floor(255 * Math.random());
  const b = Math.floor(255 * Math.random());
  return `rgb(${r},${g},${b})`;
}
