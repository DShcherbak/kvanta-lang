/**
 * Tests for drawScript() in canvas-runtime.js.
 *
 * The DOM (canvas + logs elements) and the canvas 2D context mock are set up
 * in tests/setup.js which runs before any module is imported.
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { drawScript, cancelNow } from '../../web/canvas-runtime.js';

const ctx = globalThis.__mockCtx;

function clearMocks() {
  for (const v of Object.values(ctx)) {
    if (typeof v?.mockClear === 'function') v.mockClear();
  }
}

beforeEach(() => {
  clearMocks();
  cancelNow(false);
});

// ---------------------------------------------------------------------------
// circle
// ---------------------------------------------------------------------------
describe('drawScript – circle', () => {
  it('calls ctx.arc once', () => {
    drawScript(['circle 500 500 100']);
    expect(ctx.arc).toHaveBeenCalledOnce();
  });

  it('passes correct centre and radius to arc', () => {
    drawScript(['circle 200 300 50']);
    const [cx, cy, r] = ctx.arc.mock.calls[0];
    expect(cx).toBe(200);
    expect(cy).toBe(300);
    expect(r).toBe(50);
  });

  it('strokes by default (no fill option)', () => {
    drawScript(['circle 500 500 100']);
    expect(ctx.stroke).toHaveBeenCalled();
    expect(ctx.fill).not.toHaveBeenCalled();
  });

  it('fills when fill option is provided', () => {
    drawScript(['circle 500 500 100 fill=red']);
    expect(ctx.fill).toHaveBeenCalled();
  });

  it('accepts % coordinates', () => {
    drawScript(['circle 50% 50% 10%']);
    const [cx, cy, r] = ctx.arc.mock.calls[0];
    expect(cx).toBe(500);
    expect(cy).toBe(500);
    expect(r).toBe(100);
  });
});

// ---------------------------------------------------------------------------
// rectangle
// ---------------------------------------------------------------------------
describe('drawScript – rectangle', () => {
  it('calls fillRect when fill option is present', () => {
    drawScript(['rectangle 100 100 300 300 fill=blue']);
    expect(ctx.fillRect).toHaveBeenCalled();
  });

  it('calls strokeRect when no fill is given', () => {
    drawScript(['rectangle 100 100 300 300']);
    expect(ctx.strokeRect).toHaveBeenCalled();
    expect(ctx.fillRect).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// line
// ---------------------------------------------------------------------------
describe('drawScript – line', () => {
  it('calls moveTo and lineTo with the correct coordinates', () => {
    drawScript(['line 0 0 100 200']);
    expect(ctx.moveTo).toHaveBeenCalledWith(0, 0);
    expect(ctx.lineTo).toHaveBeenCalledWith(100, 200);
  });

  it('calls stroke', () => {
    drawScript(['line 0 0 500 500']);
    expect(ctx.stroke).toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// background / bg
// ---------------------------------------------------------------------------
describe('drawScript – background', () => {
  it('bg alias clears the canvas', () => {
    drawScript(['bg red']);
    expect(ctx.fillRect).toHaveBeenCalled();
  });

  it('background command clears the canvas', () => {
    drawScript(['background #001122']);
    expect(ctx.fillRect).toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// clear
// ---------------------------------------------------------------------------
describe('drawScript – clear', () => {
  it('calls fillRect (resets canvas to default color)', () => {
    drawScript(['clear']);
    expect(ctx.fillRect).toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// empty / comment lines
// ---------------------------------------------------------------------------
describe('drawScript – blank and comment lines', () => {
  it('ignores empty lines', () => {
    drawScript(['', '   ', 'circle 500 500 50']);
    expect(ctx.arc).toHaveBeenCalledOnce();
  });

  it('ignores // comment lines', () => {
    drawScript(['// this is a comment', 'circle 500 500 50']);
    expect(ctx.arc).toHaveBeenCalledOnce();
  });
});

// ---------------------------------------------------------------------------
// cancellation
// ---------------------------------------------------------------------------
describe('drawScript – cancellation', () => {
  it('skips all commands when already cancelled', () => {
    cancelNow(true);
    drawScript(['circle 500 500 100', 'rectangle 0 0 100 100']);
    expect(ctx.arc).not.toHaveBeenCalled();
    expect(ctx.fillRect).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// unknown commands
// ---------------------------------------------------------------------------
describe('drawScript – unknown commands', () => {
  it('silently ignores unrecognised commands', () => {
    expect(() => drawScript(['unknowncommand 1 2 3'])).not.toThrow();
  });

  it('continues processing after an unknown command', () => {
    drawScript(['unknowncommand 0 0 0', 'circle 500 500 50']);
    expect(ctx.arc).toHaveBeenCalledOnce();
  });
});

// ---------------------------------------------------------------------------
// multiple commands in one script
// ---------------------------------------------------------------------------
describe('drawScript – multiple commands', () => {
  it('processes each line in order', () => {
    drawScript([
      'circle 100 100 50',
      'circle 200 200 50',
      'circle 300 300 50',
    ]);
    expect(ctx.arc).toHaveBeenCalledTimes(3);
  });
});
