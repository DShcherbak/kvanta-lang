import { describe, it, expect } from 'vitest';
import { toPx, tokenize, deg2rad, randomColorString } from '../../web/canvas-utils.js';

// ---------------------------------------------------------------------------
// toPx
// ---------------------------------------------------------------------------
describe('toPx', () => {
  it('passes through a numeric literal unchanged', () => {
    expect(toPx(100, 'x')).toBe(100);
    expect(toPx(0, 'y')).toBe(0);
  });

  it('coerces a numeric string to a number', () => {
    expect(toPx('42', 'x')).toBe(42);
    expect(toPx('0', 'y')).toBe(0);
  });

  it('converts % on the x axis relative to canvas width (1000)', () => {
    expect(toPx('0%', 'x')).toBe(0);
    expect(toPx('10%', 'x')).toBe(100);
    expect(toPx('50%', 'x')).toBe(500);
    expect(toPx('100%', 'x')).toBe(1000);
  });

  it('converts % on the y axis relative to canvas height (1000)', () => {
    expect(toPx('25%', 'y')).toBe(250);
    expect(toPx('75%', 'y')).toBe(750);
  });

  it('handles fractional percentages', () => {
    expect(toPx('12.5%', 'x')).toBeCloseTo(125);
  });
});

// ---------------------------------------------------------------------------
// tokenize
// ---------------------------------------------------------------------------
describe('tokenize', () => {
  it('splits on single spaces', () => {
    expect(tokenize('circle 50 50 20')).toEqual(['circle', '50', '50', '20']);
  });

  it('collapses multiple spaces between tokens', () => {
    expect(tokenize('line  10  20  30  40')).toEqual(['line', '10', '20', '30', '40']);
  });

  it('strips leading and trailing whitespace', () => {
    expect(tokenize('  circle 50  ')).toEqual(['circle', '50']);
  });

  it('treats tab characters as whitespace', () => {
    expect(tokenize('circle\t50\t50')).toEqual(['circle', '50', '50']);
  });

  it('returns empty array for an empty string', () => {
    expect(tokenize('')).toEqual([]);
  });

  it('returns empty array for whitespace-only input', () => {
    expect(tokenize('   ')).toEqual([]);
  });

  it('handles a single token', () => {
    expect(tokenize('clear')).toEqual(['clear']);
  });
});

// ---------------------------------------------------------------------------
// deg2rad
// ---------------------------------------------------------------------------
describe('deg2rad', () => {
  it('converts 0° to 0 rad', () => {
    expect(deg2rad(0)).toBe(0);
  });

  it('converts 90° to π/2', () => {
    expect(deg2rad(90)).toBeCloseTo(Math.PI / 2);
  });

  it('handles negative angles', () => {
    expect(deg2rad(-90)).toBeCloseTo(-Math.PI / 2);
  });

  it('handles angles > 360', () => {
    expect(deg2rad(720)).toBeCloseTo(4 * Math.PI);
  });
});

// ---------------------------------------------------------------------------
// randomColorString
// ---------------------------------------------------------------------------
describe('randomColorString', () => {
  it('returns a string matching rgb(r,g,b) format', () => {
    expect(randomColorString()).toMatch(/^rgb\(\d{1,3},\d{1,3},\d{1,3}\)$/);
  });

  it('returns each channel in the valid byte range [0, 254]', () => {
    // Run several times to reduce the chance of a lucky miss.
    for (let i = 0; i < 50; i++) {
      const c = randomColorString();
      const [r, g, b] = c.match(/\d+/g).map(Number);
      expect(r).toBeGreaterThanOrEqual(0);
      expect(r).toBeLessThanOrEqual(254); // Math.floor(255 * random) ≤ 254
      expect(g).toBeGreaterThanOrEqual(0);
      expect(g).toBeLessThanOrEqual(254);
      expect(b).toBeGreaterThanOrEqual(0);
      expect(b).toBeLessThanOrEqual(254);
    }
  });

  it('produces different values across calls (probabilistic)', () => {
    const colors = new Set(Array.from({ length: 20 }, () => randomColorString()));
    // With 256^3 possible colours, 20 calls should almost never repeat.
    expect(colors.size).toBeGreaterThan(1);
  });
});
