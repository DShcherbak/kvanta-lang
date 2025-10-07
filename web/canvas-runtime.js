const logEl  = document.getElementById('logs');
const drawCanvas = document.getElementById('canvas');
const drawCtx = drawCanvas.getContext('2d', { alpha: false });
const bufferCanvas = document.createElement('canvas')
bufferCanvas.width = 1000;
  bufferCanvas.height = 1000;
const ctx    = bufferCanvas.getContext('2d', { alpha: false });
let isAnimation = false;
let isCancelled = false;
// FIXED SIZE: 1000x1000 logical pixels (scaled for HiDPI once)
const CANVAS_W = 1000, CANVAS_H = 1000;
const DPR = Math.max(1, Math.min(3, window.devicePixelRatio || 1));

let randomColors = [];
let isSafari = false;

drawCanvas.width  = Math.floor(CANVAS_W * DPR);
drawCanvas.height = Math.floor(CANVAS_H * DPR);
bufferCanvas.width = drawCanvas.width;
bufferCanvas.height = drawCanvas.height ;
ctx.setTransform(DPR, 0, 0, DPR, 0, 0);

export function log(text) {
  if (typeof text !== 'string') text = String(text);
  logEl.textContent = text;
}

export function setup() {
  randomColors = [];
  clearCanvas();
}

export function checkIsCancelled() {
  return isCancelled;
}

export function cancelNow(value = true) {
  isCancelled = value;
  isAnimation = false;
}

export function setIsSafari(value) {
  isSafari = value;
}


function clearCanvas(color = '#0a0f1f') {
  ctx.save(); ctx.setTransform(1,0,0,1,0,0);
  ctx.clearRect(0, 0, drawCanvas.width, drawCanvas.height);
  ctx.restore();

  ctx.save(); ctx.fillStyle = color;
  ctx.fillRect(0, 0, CANVAS_W, CANVAS_H);
  ctx.restore();
}

function toPx(val, axis) {
  if (typeof val === 'string' && val.endsWith('%')) {
    const p = parseFloat(val) / 100;
    return (axis === 'x' ? CANVAS_W : CANVAS_H) * p;
  }
  return +val;
}

function randomColorString() {
  const r = Math.floor(255 * Math.random());
  const g = Math.floor(255 * Math.random());
  const b = Math.floor(255 * Math.random());
  return `rgb(${r},${g},${b})`;
}

const deg2rad = d => (d * Math.PI) / 180;
function applyStyle(opts){ ctx.lineWidth = opts.width ?? 1; if (opts.stroke) ctx.strokeStyle = opts.stroke; if (opts.fill) ctx.fillStyle = opts.fill; }
function parseOptions(tokens, startIdx){ 
    const o={}; for(let i=startIdx;i<tokens.length;i++){ 
        const t=tokens[i], eq=t.indexOf('='); 
          if(eq>0){ 
            let k=t.slice(0,eq), v=t.slice(eq+1); 
            if (v.startsWith('RandomColor')) {
              let idx = parseInt(v.slice(11));
              if (isNaN(idx)) {
                randomColors.push(randomColorString());
                idx = randomColors.length - 1;
              }
              while (idx >= randomColors.length) {
                randomColors.push(randomColorString());
              }
              v = randomColors[idx];
            }
            if(k==='width') o.width=Number(v); 
            else if(k==='stroke') o.stroke=v; 
            else if(k==='fill') o.fill=v; 
            else if(k==='ccw') o.ccw=/^(1|true|yes)$/i.test(v);
          } 
    }
    return o; 
}
function tokenize(line){ return line.trim().split(/\s+/).filter(Boolean); }

function drawCircle(cx, cy, r, o){ ctx.beginPath(); ctx.arc(toPx(cx,'x'), toPx(cy,'y'), toPx(r,'x'), 0, Math.PI*2); if(o.fill) ctx.fill(); if(o.stroke||!o.fill) ctx.stroke(); }
function drawRect(x,y,w,h,o){ const X=toPx(x,'x'),Y=toPx(y,'y'),W=toPx(w,'x'),H=toPx(h,'y'); if(o.fill) ctx.fillRect(X,Y,W-X,H-Y); if(o.stroke||!o.fill) ctx.strokeRect(X,Y,W-X,H-Y); }
function drawLine(x1,y1,x2,y2,o){ ctx.beginPath(); ctx.moveTo(toPx(x1,'x'), toPx(y1,'y')); ctx.lineTo(toPx(x2,'x'), toPx(y2,'y')); ctx.stroke(); }
function drawPolygon(nums,o){ if(nums.length<4) return; ctx.beginPath(); ctx.moveTo(toPx(nums[0],'x'), toPx(nums[1],'y')); for(let i=2;i<nums.length;i+=2) ctx.lineTo(toPx(nums[i],'x'), toPx(nums[i+1],'y')); ctx.closePath(); if(o.fill) {ctx.fill()}; if(o.stroke||!o.fill) ctx.stroke(); }
function drawArc(cx,cy,r,a0,a1,ccw,o){ ctx.beginPath(); ctx.arc(toPx(cx,'x'), toPx(cy,'y'), toPx(r,'x'), deg2rad(a0), deg2rad(a1), !!ccw); if(o.fill) ctx.fill(); if(o.stroke||!o.fill) ctx.stroke(); }

export function drawScript(script, should_draw_frame=false){
  ctx.save(); ctx.lineJoin='round'; ctx.lineCap='round';
  const lines = String(script||'').split(/,/);
  //bufferCanvas.width = drawCanvas.width;
  //bufferCanvas.height = drawCanvas.height;
  for (const raw of lines) {
    if (isCancelled) { return; }
    const line = raw.trim();
    if (!line || line.startsWith('//')) continue;
    const tok = tokenize(line); if (!tok.length) continue;
    const cmd = tok[0].toLowerCase();
    try {
      switch (cmd) {
        case 'circle': { const [_, cx, cy, r] = tok; const o=parseOptions(tok,4); applyStyle(o); drawCircle(cx,cy,r,o); break; }
        case 'rectangle': { const [_, x, y, w, h] = tok; const o=parseOptions(tok,5); applyStyle(o); drawRect(x,y,w,h,o); break; }
        case 'line': { const [_, x1, y1, x2, y2] = tok; const o=parseOptions(tok,5); applyStyle(o); drawLine(x1,y1,x2,y2,o); break; }
        case 'polygon': { const nums=[]; let i=1; for(;i<tok.length;i++){ if(tok[i].includes('=')) break; nums.push(Number(tok[i])); } const o=parseOptions(tok,i); applyStyle(o); drawPolygon(nums,o); break; }
        case 'arc': { const [_, cx, cy, r, a0, a1] = tok; const o=parseOptions(tok,6); applyStyle(o); drawArc(cx,cy,r,Number(a0),Number(a1),!!o.ccw,o); break; }
        case 'bg': case 'background': { const color = tok[1] || '#0a0f1f'; clearCanvas(color); break; }
        case 'animate': {isAnimation = true; break;}
        case 'clear': {clearCanvas(); break; }
        case 'error': {alert("Error: " + raw); break; }
        case 'print': { const msg = raw.slice(5).trim(); console.log("Print:" + msg); break; }
        default: /* ignore unknown */ break;
      }
    } catch (e) { console.warn('Error:', line, e); }
  }
  ctx.restore();
  if (!isAnimation || should_draw_frame) {
    drawCtx.clearRect(0, 0, drawCanvas.width, drawCanvas.height);
    drawCtx.drawImage(bufferCanvas, 0, 0);
    if (isSafari) {
      drawCanvas.getContext("2d").getImageData(0, 0, 1, 1); // Force repaint
    }
  }
}

export function resizeCanvases() {
  
  const rect = drawCanvas.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;

  // Match canvas internal size to actual visible size * device pixel ratio
  const width = Math.floor(rect.width * dpr);
  const height = Math.floor(rect.height * dpr);

  if (drawCanvas.width !== width || drawCanvas.height !== height) {
    drawCanvas.width = width;
    drawCanvas.height = height;
    bufferCanvas.width = width;
        bufferCanvas.height = height;

  }
}
