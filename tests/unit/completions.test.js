/**
 * Tests for the autocompletion data in quanta-support.ts.
 *
 * rawCompletionItems is a plain array with no DOM or canvas dependencies,
 * so no special setup is needed beyond Vitest loading the module.
 */
import { describe, it, expect } from 'vitest';
import { rawCompletionItems } from '../../web/quanta-support.ts';

const labels = rawCompletionItems.map(item => item.label);
const byLabel = Object.fromEntries(rawCompletionItems.map(item => [item.label, item]));

// ---------------------------------------------------------------------------
// Basic shape
// ---------------------------------------------------------------------------
describe('rawCompletionItems', () => {
  it('is a non-empty array', () => {
    expect(Array.isArray(rawCompletionItems)).toBe(true);
    expect(rawCompletionItems.length).toBeGreaterThan(0);
  });

  it('every entry has a label (string) and type (string)', () => {
    for (const item of rawCompletionItems) {
      expect(typeof item.label).toBe('string');
      expect(item.label.length).toBeGreaterThan(0);
      expect(typeof item.type).toBe('string');
    }
  });

  it('has no duplicate labels', () => {
    const unique = new Set(labels);
    expect(unique.size).toBe(rawCompletionItems.length);
  });
});

// ---------------------------------------------------------------------------
// Type keywords
// ---------------------------------------------------------------------------
describe('type keywords', () => {
  it.each(['bool', 'int', 'float', 'Color'])(
    '"%s" is present as a keyword',
    label => {
      expect(labels).toContain(label);
      expect(byLabel[label].type).toBe('keyword');
    }
  );
});

// ---------------------------------------------------------------------------
// Drawing functions
// ---------------------------------------------------------------------------
describe('drawing functions', () => {
  it.each(['circle', 'rectangle', 'line', 'setLineColor', 'setFigureColor'])(
    '"%s" is present as a function',
    label => {
      expect(labels).toContain(label);
      expect(byLabel[label].type).toBe('function');
    }
  );
});

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------
describe('colour keywords', () => {
  const expectedColors = [
    'Red', 'Green', 'Blue', 'Yellow', 'Orange',
    'Pink', 'Purple', 'Brown', 'Cyan',
    'Black', 'Gray', 'White',
  ];

  it.each(expectedColors)('"%s" is present as a keyword', label => {
    expect(labels).toContain(label);
    expect(byLabel[label].type).toBe('keyword');
  });

  it('includes dark/light variants', () => {
    expect(labels).toContain('DarkRed');
    expect(labels).toContain('LightBlue');
    expect(labels).toContain('DarkGreen');
    expect(labels).toContain('LightGray');
  });

  it('includes special colour constants', () => {
    expect(labels).toContain('Random');
    expect(labels).toContain('Transparent');
    expect(labels).toContain('Background');
  });
});
