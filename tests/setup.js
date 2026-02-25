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
