import { vi } from 'vitest';

// Populate the DOM that canvas-runtime.js queries at module load time.
const canvas = document.createElement('canvas');
canvas.id = 'canvas';
canvas.width = 1000;
canvas.height = 1000;
document.body.appendChild(canvas);

const logs = document.createElement('div');
logs.id = 'logs';
document.body.appendChild(logs);

// --------------------------------------------------------------------------
// DOM elements required by web/main.js at module load time.
// --------------------------------------------------------------------------

const runBtn = document.createElement('button');
runBtn.id = 'runBtn';
document.body.appendChild(runBtn);

const editorDiv = document.createElement('div');
editorDiv.id = 'editor';
document.body.appendChild(editorDiv);

const resizer = document.createElement('div');
resizer.id = 'resizer';
document.body.appendChild(resizer);

const panesDiv = document.createElement('div');
panesDiv.className = 'panes';
document.body.appendChild(panesDiv);

const downloadBtn = document.createElement('button');
downloadBtn.id = 'downloadBtn';
document.body.appendChild(downloadBtn);

const loadBtn = document.createElement('button');
loadBtn.id = 'loadBtn';
document.body.appendChild(loadBtn);

const saveBtn = document.createElement('button');
saveBtn.id = 'saveBtn';
document.body.appendChild(saveBtn);

const fileInput = document.createElement('input');
fileInput.type = 'file';
fileInput.id = 'fileInput';
document.body.appendChild(fileInput);

// ResizeObserver stub – jsdom does not provide this API but CodeMirror 6
// checks for its existence when mounting an EditorView.
if (typeof globalThis.ResizeObserver === 'undefined') {
  globalThis.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}

// Mock canvas 2D context (jsdom does not implement CanvasRenderingContext2D).
const mockCtx = {
  save: vi.fn(),
  restore: vi.fn(),
  clearRect: vi.fn(),
  fillRect: vi.fn(),
  strokeRect: vi.fn(),
  beginPath: vi.fn(),
  arc: vi.fn(),
  fill: vi.fn(),
  stroke: vi.fn(),
  moveTo: vi.fn(),
  lineTo: vi.fn(),
  closePath: vi.fn(),
  drawImage: vi.fn(),
  setTransform: vi.fn(),
  getImageData: vi.fn(() => ({ data: [] })),
  lineJoin: '',
  lineCap: '',
  lineWidth: 1,
  strokeStyle: '',
  fillStyle: '',
};

HTMLCanvasElement.prototype.getContext = () => mockCtx;
HTMLCanvasElement.prototype.getBoundingClientRect = () => ({
  width: 500, height: 500, top: 0, left: 0, right: 500, bottom: 500,
});

// Expose on globalThis so drawing-commands tests can inspect calls.
globalThis.__mockCtx = mockCtx;
