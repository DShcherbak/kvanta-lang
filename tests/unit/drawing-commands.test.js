/**
 * Tests for drawScript() in canvas-runtime.js.
 *
 * The DOM (canvas + logs elements) and the canvas 2D context mock are set up
 * in tests/setup.js which runs before any module is imported.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
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

  it('ignores // a valid lines under the comment', () => {
    drawScript(['//circle 500 500 50']);
    expect(ctx.arc).not.toHaveBeenCalled();
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

// ---------------------------------------------------------------------------
// polygon
// ---------------------------------------------------------------------------
describe('drawScript – polygon', () => {
  it('calls beginPath and closePath', () => {
    drawScript(['polygon 0 0 100 0 50 100']);
    expect(ctx.beginPath).toHaveBeenCalled();
    expect(ctx.closePath).toHaveBeenCalled();
  });

  it('calls moveTo for first point and lineTo for subsequent points', () => {
    drawScript(['polygon 10 20 30 40 50 60']);
    expect(ctx.moveTo).toHaveBeenCalledWith(10, 20);
    expect(ctx.lineTo).toHaveBeenCalledWith(30, 40);
    expect(ctx.lineTo).toHaveBeenCalledWith(50, 60);
  });

  it('strokes by default', () => {
    drawScript(['polygon 0 0 100 0 50 100']);
    expect(ctx.stroke).toHaveBeenCalled();
    expect(ctx.fill).not.toHaveBeenCalled();
  });

  it('fills when fill option is provided', () => {
    drawScript(['polygon 0 0 100 0 50 100 fill=green']);
    expect(ctx.fill).toHaveBeenCalled();
  });

  it('does nothing when fewer than 4 numbers (less than 2 points) are given', () => {
    drawScript(['polygon 100 200']);
    expect(ctx.beginPath).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// arc
// ---------------------------------------------------------------------------
describe('drawScript – arc', () => {
  it('calls ctx.arc once', () => {
    drawScript(['arc 500 500 100 0 90']);
    expect(ctx.arc).toHaveBeenCalledOnce();
  });

  it('passes correct centre and radius', () => {
    drawScript(['arc 200 300 50 0 180']);
    const [cx, cy, r] = ctx.arc.mock.calls[0];
    expect(cx).toBe(200);
    expect(cy).toBe(300);
    expect(r).toBe(50);
  });

  it('converts start and end angles from degrees to radians', () => {
    drawScript(['arc 500 500 100 0 180']);
    const [, , , a0, a1] = ctx.arc.mock.calls[0];
    expect(a0).toBeCloseTo(0);
    expect(a1).toBeCloseTo(Math.PI);
  });

  it('strokes by default', () => {
    drawScript(['arc 500 500 100 0 90']);
    expect(ctx.stroke).toHaveBeenCalled();
    expect(ctx.fill).not.toHaveBeenCalled();
  });

  it('fills when fill option is provided', () => {
    drawScript(['arc 500 500 100 0 90 fill=red']);
    expect(ctx.fill).toHaveBeenCalled();
  });

  it('passes ccw=true when ccw option is set', () => {
    drawScript(['arc 500 500 100 0 90 ccw=true']);
    const [, , , , , ccw] = ctx.arc.mock.calls[0];
    expect(ccw).toBe(true);
  });

  it('passes ccw=false when ccw option is absent', () => {
    drawScript(['arc 500 500 100 0 90']);
    const [, , , , , ccw] = ctx.arc.mock.calls[0];
    expect(ccw).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// style options
// ---------------------------------------------------------------------------
describe('drawScript – style options', () => {
  it('width= sets lineWidth on the context', () => {
    drawScript(['circle 500 500 100 width=5']);
    expect(ctx.lineWidth).toBe(5);
  });

  it('stroke= sets strokeStyle on the context', () => {
    drawScript(['circle 500 500 100 stroke=blue']);
    expect(ctx.strokeStyle).toBe('blue');
  });

  it('fill= sets fillStyle on the context', () => {
    drawScript(['circle 500 500 100 fill=red']);
    expect(ctx.fillStyle).toBe('red');
  });

  it('calls both fill() and stroke() when both fill= and stroke= are provided', () => {
    drawScript(['circle 500 500 100 fill=red stroke=blue']);
    expect(ctx.fill).toHaveBeenCalled();
    expect(ctx.stroke).toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// rectangle – coordinate correctness
// ---------------------------------------------------------------------------
describe('drawScript – rectangle coordinates', () => {
  it('computes width and height from corner coordinates for fillRect', () => {
    drawScript(['rectangle 100 100 300 300 fill=blue']);
    expect(ctx.fillRect).toHaveBeenCalledWith(100, 100, 200, 200);
  });

  it('computes width and height from corner coordinates for strokeRect', () => {
    drawScript(['rectangle 50 50 250 150']);
    expect(ctx.strokeRect).toHaveBeenCalledWith(50, 50, 200, 100);
  });
});

// ---------------------------------------------------------------------------
// print
// ---------------------------------------------------------------------------
describe('drawScript – print', () => {
  it('logs the message to the console', () => {
    const spy = vi.spyOn(console, 'log').mockImplementation(() => {});
    drawScript(['print hello world']);
    expect(spy).toHaveBeenCalledWith('Print:hello world');
    spy.mockRestore();
  });

  it('does not throw', () => {
    expect(() => drawScript(['print hello'])).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// error
// ---------------------------------------------------------------------------
describe('drawScript – error', () => {
  it('calls window.alert with the error message', () => {
    const spy = vi.spyOn(window, 'alert').mockImplementation(() => {});
    drawScript(['error something went wrong']);
    expect(spy).toHaveBeenCalled();
    spy.mockRestore();
  });
});

// ---------------------------------------------------------------------------
// compositing (drawImage)
// ---------------------------------------------------------------------------
describe('drawScript – compositing', () => {
  it('composites to the visible canvas after a normal script', () => {
    drawScript(['circle 500 500 100']);
    expect(ctx.drawImage).toHaveBeenCalled();
  });

  it('skips compositing in animation mode (no should_draw_frame)', () => {
    drawScript(['animate', 'circle 500 500 100']);
    expect(ctx.drawImage).not.toHaveBeenCalled();
  });

  it('composites when should_draw_frame=true in animation mode', () => {
    drawScript(['animate']);
    ctx.drawImage.mockClear();
    drawScript(['circle 500 500 100'], true);
    expect(ctx.drawImage).toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// RandomColor
// ---------------------------------------------------------------------------
describe('drawScript – RandomColor', () => {
  it('same RandomColor index produces the same color on repeated calls', () => {
    drawScript(['circle 100 100 50 fill=RandomColor0']);
    const firstColor = ctx.fillStyle;
    clearMocks();
    drawScript(['circle 200 200 50 fill=RandomColor0']);
    expect(ctx.fillStyle).toBe(firstColor);
  });

  it('different RandomColor indices produce independently stored colors', () => {
    drawScript(['circle 100 100 50 fill=RandomColor5']);
    const color5 = ctx.fillStyle;
    clearMocks();
    drawScript(['circle 200 200 50 fill=RandomColor6']);
    const color6 = ctx.fillStyle;
    // Both are valid CSS color strings (rgb format)
    expect(color5).toMatch(/^rgb\(/);
    expect(color6).toMatch(/^rgb\(/);
  });

  it('plain RandomColor (no index) appends a new entry each call', () => {
    // Each plain `RandomColor` token pushes a fresh random color onto the palette.
    // Collect the fillStyle produced by 20 successive calls; at least two must
    // differ (the probability of 20 independent uniform rgb values all being
    // identical by chance is ~(1/255^3)^19 ≈ 10^-137).
    const colors = [];
    for (let i = 0; i < 20; i++) {
      clearMocks();
      drawScript(['circle 100 100 50 fill=RandomColor']);
      colors.push(ctx.fillStyle);
    }
    const unique = new Set(colors);
    expect(unique.size).toBeGreaterThan(1);
  });
});
