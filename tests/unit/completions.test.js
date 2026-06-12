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