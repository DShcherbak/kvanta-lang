/**
 * Tests for the utility functions and event handlers in web/main.js.
 *
 * Strategy
 * --------
 * • The WASM compiler is mocked (it cannot run in Node/jsdom).
 * • canvas-runtime is mocked so we can spy on calls like `setup` and `cancelNow`.
 * • All required DOM elements (runBtn, editor, resizer, …) are pre-created
 *   in tests/setup.js, which runs before the module is imported.
 * • Exported pure utilities (sleep, fontSizeTheme, downloadFile, alertError,
 *   showError, showOk) are imported and tested directly.
 * • DOM event handlers are exercised by dispatching events on the real elements.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

// ---------------------------------------------------------------------------
// Module mocks – Vitest hoists these before any imports.
// ---------------------------------------------------------------------------

vi.mock('../../quanta-lang/pkg/quanta_lang.js', () => {
  const mockRuntime = {
    execute: vi.fn(),
    execute_key: vi.fn(),
    execute_mouse: vi.fn(),
    get_commands: vi.fn(() => []),
    get_runtime_error: vi.fn(),
  };
  return {
    default: vi.fn().mockResolvedValue(undefined), // initWasm
    Compiler: {
      new: vi.fn(() => ({
        compile_code: vi.fn().mockResolvedValue({
          error_code: 0,
          get_error: vi.fn(),
          get_runtime: vi.fn(() => mockRuntime),
        }),
      })),
    },
  };
});

vi.mock('../../web/canvas-runtime.js', () => ({
  drawScript: vi.fn(),
  setup: vi.fn(),
  checkIsCancelled: vi.fn(() => false),
  cancelNow: vi.fn(),
  setIsSafari: vi.fn(),
}));

// ---------------------------------------------------------------------------
// Imports (after mocks are registered)
// ---------------------------------------------------------------------------

import { EditorState } from '@codemirror/state';

import {
  sleep,
  fontSizeTheme,
  downloadFile,
  alertError,
  showError,
  showOk,
} from '../../web/main.js';

import { setup, cancelNow } from '../../web/canvas-runtime.js';

// ============================================================
// sleep
// ============================================================

describe('sleep', () => {
  it('returns a Promise', () => {
    expect(sleep(0)).toBeInstanceOf(Promise);
  });

  it('resolves to undefined', async () => {
    await expect(sleep(0)).resolves.toBeUndefined();
  });

  it('does not resolve before the delay elapses', async () => {
    vi.useFakeTimers();
    let resolved = false;
    sleep(500).then(() => { resolved = true; });
    vi.advanceTimersByTime(499);
    await Promise.resolve(); // flush microtasks
    expect(resolved).toBe(false);
    vi.useRealTimers();
  });

  it('resolves after the full delay', async () => {
    vi.useFakeTimers();
    const p = sleep(500);
    vi.advanceTimersByTime(500);
    await expect(p).resolves.toBeUndefined();
    vi.useRealTimers();
  });
});

// ============================================================
// fontSizeTheme
// ============================================================

describe('fontSizeTheme', () => {
  it('returns a truthy CodeMirror extension', () => {
    expect(fontSizeTheme(16)).toBeTruthy();
  });

  it('returns an object', () => {
    expect(typeof fontSizeTheme(18)).toBe('object');
  });

  it('returns a distinct value for each call', () => {
    // EditorView.theme() creates a new StyleModule each time
    expect(fontSizeTheme(14)).not.toBe(fontSizeTheme(20));
  });

  it('does not throw for minimum-sized fonts', () => {
    expect(() => fontSizeTheme(1)).not.toThrow();
  });

  it('does not throw for large fonts', () => {
    expect(() => fontSizeTheme(72)).not.toThrow();
  });
});

// ============================================================
// downloadFile
// ============================================================

describe('downloadFile', () => {
  let capturedAnchor;

  beforeEach(() => {
    capturedAnchor = undefined;
    global.URL.createObjectURL = vi.fn(() => 'blob:fake-url');
    global.URL.revokeObjectURL = vi.fn();

    const origCreate = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag) => {
      const el = origCreate(tag);
      if (tag === 'a') {
        capturedAnchor = el;
        el.click = vi.fn();
      }
      return el;
    });
  });

  afterEach(() => vi.restoreAllMocks());

  it('calls URL.createObjectURL with a Blob', () => {
    downloadFile('test.txt', 'hello world');
    expect(URL.createObjectURL).toHaveBeenCalledWith(expect.any(Blob));
  });

  it('creates a plain-text Blob', () => {
    downloadFile('test.txt', 'hello');
    const blob = URL.createObjectURL.mock.calls[0][0];
    expect(blob.type).toBe('text/plain');
  });

  it('sets the anchor href to the object URL', () => {
    downloadFile('file.txt', 'data');
    expect(capturedAnchor.href).toBe('blob:fake-url');
  });

  it('sets the correct download filename', () => {
    downloadFile('program.quanta', 'code');
    expect(capturedAnchor.download).toBe('program.quanta');
  });

  it('triggers a click on the anchor element', () => {
    downloadFile('file.txt', 'content');
    expect(capturedAnchor.click).toHaveBeenCalled();
  });

  it('revokes the object URL after clicking', () => {
    downloadFile('file.txt', 'content');
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:fake-url');
  });
});

// ============================================================
// alertError
// ============================================================

describe('alertError', () => {
  let logSpy, alertSpy;

  const makeErr = (msg = 'bad token', sr = 2, sc = 4, er = 2, ec = 9) => ({
    start_row: sr,
    start_column: sc,
    end_row: er,
    end_column: ec,
    get_error_message: () => msg,
  });

  beforeEach(() => {
    logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});
  });

  afterEach(() => {
    logSpy.mockRestore();
    alertSpy.mockRestore();
  });

  it('calls console.log once', () => {
    alertError(makeErr());
    expect(logSpy).toHaveBeenCalledOnce();
  });

  it('includes the error message in the console output', () => {
    alertError(makeErr('undefined variable'));
    expect(logSpy).toHaveBeenCalledWith(
      expect.stringContaining('undefined variable'),
    );
  });

  it('includes row and column in the console output', () => {
    alertError(makeErr('x', 3, 7, 3, 12));
    expect(logSpy).toHaveBeenCalledWith(expect.stringContaining('3:7'));
  });

  it('calls window.alert once', () => {
    alertError(makeErr());
    expect(alertSpy).toHaveBeenCalledOnce();
  });

  it('includes the error message in the alert', () => {
    alertError(makeErr('syntax error'));
    expect(alertSpy).toHaveBeenCalledWith(
      expect.stringContaining('syntax error'),
    );
  });

  it('includes the start row:column in the alert', () => {
    alertError(makeErr('e', 5, 12, 5, 15));
    expect(alertSpy).toHaveBeenCalledWith(expect.stringContaining('5:12'));
  });

  it('includes the end row:column in the alert', () => {
    alertError(makeErr('e', 1, 0, 1, 4));
    expect(alertSpy).toHaveBeenCalledWith(expect.stringContaining('1:4'));
  });
});

// ============================================================
// showError
// ============================================================

describe('showError', () => {
  it('calls editor.dispatch once', () => {
    const state = EditorState.create({ doc: 'hello\nworld\n' });
    const dispatchSpy = vi.fn();
    const err = {
      start_row: 1,
      start_column: 1,
      end_row: 1,
      end_column: 4,
      get_error_message: () => 'test',
    };
    showError({ state, dispatch: dispatchSpy }, err);
    expect(dispatchSpy).toHaveBeenCalledOnce();
  });

  it('clamps column to line boundaries when column exceeds line length', () => {
    // "hi\n" = line 1 from:0, to:2  →  column 999 should be clamped to 2
    const state = EditorState.create({ doc: 'hi\nbye\n' });
    const dispatchSpy = vi.fn();
    const err = {
      start_row: 1,
      start_column: 999,
      end_row: 1,
      end_column: 999,
      get_error_message: () => 'overflow',
    };
    expect(() => showError({ state, dispatch: dispatchSpy }, err)).not.toThrow();
    expect(dispatchSpy).toHaveBeenCalledOnce();
  });

  it('handles start_row = 0 (below minimum) without throwing', () => {
    const state = EditorState.create({ doc: 'code' });
    const dispatchSpy = vi.fn();
    const err = {
      start_row: 0,
      start_column: 0,
      end_row: 0,
      end_column: 2,
      get_error_message: () => 'zero row',
    };
    expect(() => showError({ state, dispatch: dispatchSpy }, err)).not.toThrow();
  });
});

// ============================================================
// showOk
// ============================================================

describe('showOk', () => {
  it('calls editor.dispatch once', () => {
    const state = EditorState.create({ doc: 'code' });
    const dispatchSpy = vi.fn();
    showOk({ state, dispatch: dispatchSpy });
    expect(dispatchSpy).toHaveBeenCalledOnce();
  });

  it('does not throw on an empty document', () => {
    const state = EditorState.create({ doc: '' });
    const dispatchSpy = vi.fn();
    expect(() => showOk({ state, dispatch: dispatchSpy })).not.toThrow();
  });
});

// ============================================================
// downloadBtn click handler
// ============================================================

describe('downloadBtn – click handler', () => {
  afterEach(() => vi.restoreAllMocks());

  it('calls window.prompt to ask for a filename', () => {
    const promptSpy = vi.spyOn(window, 'prompt').mockReturnValue(null);
    document.getElementById('downloadBtn').click();
    expect(promptSpy).toHaveBeenCalled();
  });

  it('does not create a download when prompt is cancelled', () => {
    vi.spyOn(window, 'prompt').mockReturnValue(null);
    global.URL.createObjectURL = vi.fn();
    document.getElementById('downloadBtn').click();
    expect(URL.createObjectURL).not.toHaveBeenCalled();
  });

  it('appends .quanta to a bare filename', () => {
    vi.spyOn(window, 'prompt').mockReturnValue('myprogram');
    global.URL.createObjectURL = vi.fn(() => 'blob:u');
    global.URL.revokeObjectURL = vi.fn();
    let captured = null;
    const orig = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag) => {
      const el = orig(tag);
      if (tag === 'a') { captured = el; el.click = vi.fn(); }
      return el;
    });
    document.getElementById('downloadBtn').click();
    expect(captured.download).toBe('myprogram.quanta');
  });

  it('preserves an existing .quanta extension', () => {
    vi.spyOn(window, 'prompt').mockReturnValue('code.quanta');
    global.URL.createObjectURL = vi.fn(() => 'blob:u');
    global.URL.revokeObjectURL = vi.fn();
    let captured = null;
    const orig = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag) => {
      const el = orig(tag);
      if (tag === 'a') { captured = el; el.click = vi.fn(); }
      return el;
    });
    document.getElementById('downloadBtn').click();
    expect(captured.download).toBe('code.quanta');
  });

  it('triggers a download click when a name is provided', () => {
    vi.spyOn(window, 'prompt').mockReturnValue('prog');
    global.URL.createObjectURL = vi.fn(() => 'blob:u');
    global.URL.revokeObjectURL = vi.fn();
    let anchorClicked = false;
    const orig = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag) => {
      const el = orig(tag);
      if (tag === 'a') el.click = vi.fn(() => { anchorClicked = true; });
      return el;
    });
    document.getElementById('downloadBtn').click();
    expect(anchorClicked).toBe(true);
  });
});

// ============================================================
// loadBtn click handler
// ============================================================

describe('loadBtn – click handler', () => {
  it('triggers a click on the hidden file input', () => {
    const fileInput = document.getElementById('fileInput');
    const clickSpy = vi.spyOn(fileInput, 'click').mockImplementation(() => {});
    document.getElementById('loadBtn').click();
    expect(clickSpy).toHaveBeenCalled();
    clickSpy.mockRestore();
  });
});

// ============================================================
// saveBtn click handler
// ============================================================

describe('saveBtn – click handler', () => {
  afterEach(() => vi.restoreAllMocks());

  it('calls canvas.toDataURL with JPEG format and 0.95 quality', () => {
    vi.spyOn(window, 'prompt').mockReturnValue('painting');
    const canvasEl = document.getElementById('canvas');
    const toDataURLSpy = vi.spyOn(canvasEl, 'toDataURL').mockReturnValue(
      'data:image/jpeg;base64,',
    );
    const orig = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag) => {
      const el = orig(tag);
      if (tag === 'a') el.click = vi.fn();
      return el;
    });
    document.getElementById('saveBtn').click();
    expect(toDataURLSpy).toHaveBeenCalledWith('image/jpeg', 0.95);
  });

  it('appends .jpg to the chosen filename', () => {
    vi.spyOn(window, 'prompt').mockReturnValue('myart');
    const canvasEl = document.getElementById('canvas');
    vi.spyOn(canvasEl, 'toDataURL').mockReturnValue(
      'data:image/jpeg;base64,x',
    );
    let captured = null;
    const orig = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag) => {
      const el = orig(tag);
      if (tag === 'a') { captured = el; el.click = vi.fn(); }
      return el;
    });
    document.getElementById('saveBtn').click();
    expect(captured.download).toBe('myart.jpg');
  });

  it('aborts without triggering a download when prompt is cancelled', () => {
    vi.spyOn(window, 'prompt').mockReturnValue(null);
    const canvasEl = document.getElementById('canvas');
    vi.spyOn(canvasEl, 'toDataURL').mockReturnValue('data:image/jpeg;base64,');
    let anchorClicked = false;
    const orig = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag) => {
      const el = orig(tag);
      if (tag === 'a') el.click = vi.fn(() => { anchorClicked = true; });
      return el;
    });
    document.getElementById('saveBtn').click();
    expect(anchorClicked).toBe(false);
  });
});

// ============================================================
// Pane resizer
// ============================================================

describe('Pane resizer', () => {
  let panesEl;

  beforeEach(() => {
    panesEl = document.querySelector('.panes');
    vi.spyOn(panesEl, 'getBoundingClientRect').mockReturnValue({
      width: 1000,
      top: 0,
      left: 0,
      right: 1000,
      bottom: 600,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
    // Reset drag state so tests stay independent.
    window.dispatchEvent(new MouseEvent('mouseup'));
    panesEl.style.gridTemplateColumns = '';
  });

  it('updates gridTemplateColumns while dragging', () => {
    document.getElementById('resizer').dispatchEvent(
      new MouseEvent('mousedown'),
    );
    window.dispatchEvent(new MouseEvent('mousemove', { clientX: 400 }));
    // leftWidth=400, rightWidth=1000-400-4=596
    expect(panesEl.style.gridTemplateColumns).toBe('400px 4px 596px');
  });

  it('computes the right-pane width as totalWidth − leftWidth − 4', () => {
    document.getElementById('resizer').dispatchEvent(
      new MouseEvent('mousedown'),
    );
    window.dispatchEvent(new MouseEvent('mousemove', { clientX: 200 }));
    expect(panesEl.style.gridTemplateColumns).toBe('200px 4px 796px');
  });

  it('does not update gridTemplateColumns before dragging starts', () => {
    panesEl.style.gridTemplateColumns = 'initial';
    window.dispatchEvent(new MouseEvent('mousemove', { clientX: 400 }));
    expect(panesEl.style.gridTemplateColumns).toBe('initial');
  });

  it('stops updating after mouseup', () => {
    document.getElementById('resizer').dispatchEvent(
      new MouseEvent('mousedown'),
    );
    window.dispatchEvent(new MouseEvent('mouseup'));
    window.dispatchEvent(new MouseEvent('mousemove', { clientX: 400 }));
    // gridTemplateColumns should not have been updated after mouseup
    expect(panesEl.style.gridTemplateColumns).toBe('');
  });

  it('resets the document cursor on mouseup', () => {
    document.getElementById('resizer').dispatchEvent(
      new MouseEvent('mousedown'),
    );
    expect(document.body.style.cursor).toBe('col-resize');
    window.dispatchEvent(new MouseEvent('mouseup'));
    expect(document.body.style.cursor).toBe('');
  });
});

// ============================================================
// runBtn click handler
// ============================================================

describe('runBtn – click handler', () => {
  beforeEach(() => vi.clearAllMocks());

  it('calls cancelNow(false) at the start of a run', async () => {
    document.getElementById('runBtn').click();
    await vi.waitFor(
      () => expect(cancelNow).toHaveBeenCalledWith(false),
      { timeout: 2000 },
    );
  });

  it('calls setup() on the canvas runtime', async () => {
    document.getElementById('runBtn').click();
    await vi.waitFor(
      () => expect(setup).toHaveBeenCalled(),
      { timeout: 2000 },
    );
  });
});
