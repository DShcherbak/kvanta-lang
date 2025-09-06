// CodeMirror bits (via esm.sh, no local install needed)
//import { EditorView, lineNumbers, highlightActiveLine } from "@codemirror/view";
//import { EditorState } from "@codemirror/state";
//import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
//import { indentOnInput } from "@codemirror/language";
import { oneDark } from "@codemirror/theme-one-dark";
//import { autocompletion } from "@codemirror/autocomplete";
import {EditorState} from "@codemirror/state"
import {
  EditorView, keymap, highlightSpecialChars, drawSelection,
  highlightActiveLine, dropCursor, rectangularSelection,
  crosshairCursor, lineNumbers, highlightActiveLineGutter
} from "@codemirror/view"
import {
  defaultHighlightStyle, syntaxHighlighting, indentOnInput,
  bracketMatching, foldGutter, foldKeymap
} from "@codemirror/language"
import {
  defaultKeymap, history, historyKeymap
} from "@codemirror/commands"
import {
  autocompletion, closeBrackets,
  closeBracketsKeymap, completionKeymap
} from "@codemirror/autocomplete"
// Language support (your Lezer parser compiled to quanta.js)
import { quanta } from "./quanta-support.ts";


// Canvas runtime (drawScript + utilities)
import { drawScript, clearCanvas, checkIsCancelled, cancelNow } from "./canvas-runtime.js";

// WASM glue (wasm-pack output); adjust crate name/path
import initWasm, { Compiler } from "../quanta-lang/pkg/quanta_lang.js"; 

const runBtn = document.getElementById("runBtn");

let runtime = undefined;
let isRunning = false;

// starter code
const startCode = 
`func mouse(int z, int y) {
    setFigureColor(Color::Red);
    rectangle(z, y, z+100, y+100);
    x = x + 10;
}

func keyboard(int key) {
    if (key == Key::Space) {
        setFigureColor(Color::Blue);
    } else {
      if (key == Key::A) {
          setFigureColor(Color::Black);
      } else {
          setFigureColor(Color::Yellow);
      }
    }
    x = x - 10;
}

global {
    int x = 320;
}

func main() {
   setLineColor(Color::Green);
   for i in (0..10000) {
      circle(x, 240, i % 100);
   }
   rectangle(0, 0, 100, 100);
}

`;

const editor = new EditorView({
  state: EditorState.create({
    doc: startCode,
     extensions: [
    // A line number gutter
    lineNumbers(),
    // A gutter with code folding markers
    foldGutter(),
    // Replace non-printable characters with placeholders
    highlightSpecialChars(),
    // The undo history
    history(),
    // Replace native cursor/selection with our own
    drawSelection(),
    // Show a drop cursor when dragging over the editor
    dropCursor(),
    // Allow multiple cursors/selections
    EditorState.allowMultipleSelections.of(true),
    // Re-indent lines when typing specific input
    indentOnInput(),
    // Highlight syntax with a default style
    syntaxHighlighting(defaultHighlightStyle),
    // Highlight matching brackets near cursor
    bracketMatching(),
    // Automatically close brackets
    closeBrackets(),
    // Load the autocompletion system
    autocompletion(),
    // Allow alt-drag to select rectangular regions
    rectangularSelection(),
    // Change the cursor to a crosshair when holding alt
    crosshairCursor(),
    // Style the current line specially
    highlightActiveLine(),
    // Style the gutter for current line specially
    highlightActiveLineGutter(),
    oneDark,
    quanta(),
    //keymap.of([{key: "Tab", run: acceptCompletion}]),
    // Highlight text that matches the selected text
    //highlightSelectionMatches(),
    keymap.of([
      // Closed-brackets aware backspace
      ...closeBracketsKeymap,
      // A large set of basic bindings
      ...defaultKeymap,
      // Redo/undo keys
      ...historyKeymap,
      // Code folding bindings
      ...foldKeymap,
      // Autocompletion keys
      ...completionKeymap,
      // Keys related to the linter system
      //...lintKeymap
    ])
  ]
    // extensions: [
    //   lineNumbers(),
    //   highlightActiveLine(),
    //   indentOnInput(),
    //   history(),
    //   autocompletion(),
    //   quanta(),
    //   EditorView.updateListener.of(v => { /* hooks later */ }),
    //   oneDark
    // ]
  }),
  parent: document.getElementById("editor")
});



function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function doStop() {
  (async() => {
    console.log("STOP!");
    runtime = undefined;
    cancelNow()
  })();
}

async function startExecution() {
  let res = runtime.execute();
  console.log("EXECUTION ENDED: " + res);
}

async function executeKey(key) {
  let res = runtime.execute_key(key);
  console.log("EXECUTION ENDED: " + res);
}

async function executeMouse(x, y) {
  let res = runtime.execute_mouse(x, y);
  console.log("EXECUTION ENDED: " + res);
}

// window.addEventListener('keydown', (e) => {
//   if (!runtime || !runtime.execute_key) return;

//   try {
//     console.log("Got key: " + e.key);
//     (async () => {runtime.execute_key(e.key);})(); // pass string like 'a', 'Enter', etc.
//   } catch (err) {
//     console.warn('Keyboard runtime error:', err);
//   }
// });

function doRun() {
  (async () => {
    try {
      cancelNow(false);
      isRunning = true;
      runBtn.disabled = true;
      clearCanvas();
      await initWasm();
      const src = editor.state.doc.toString();
      let compiler = Compiler.new();
      const compilation_result = await compiler.compile_code(src);   // Rust returns drawing commands (string)
      if (compilation_result.error_code != 0) {
        alert(compilation_result.get_error_message() + " at " 
            + compilation_result.start_row + ":" + compilation_result.start_column
            + " - " + compilation_result.end_row + ":" + compilation_result.end_column);
        runBtn.disabled = false;
        return;
      }
      console.log("Compiling done");
      setRunningUI();
      runtime = compilation_result.get_runtime();
      startExecution();
      let need_continue = true;
      while(need_continue) {
        if (checkIsCancelled()) { return; }
        let blocks = runtime.get_commands();
        for (let i = 0; i < blocks.length; i++) {
          if (checkIsCancelled()) { return; }
          const block = blocks[i];
          let commands = block.get_commands();
           drawScript(commands, block.should_draw_frame);
           if (block.sleep_for >= 0) {
            await sleep(block.sleep_for);
           } else {
            console.log("finish!");
            need_continue = false;
            break;
           }
        }
      }
    } catch (e) {
      console.error(e);
      alert("Error: " + (e?.message ?? String(e)));
    } finally {
      setIdleUI();
      runBtn.disabled = false;
    }
  })();
}

function setRunningUI() {
  isRunning = true;
  runBtn.textContent = 'Stop';
  runBtn.dataset.state = 'stop';
  runBtn.disabled = false;
}

function setIdleUI() {
  isRunning = false;
  runBtn.textContent = 'Run (Ctrl/Cmd+Enter)';
  runBtn.dataset.state = 'run';
  runBtn.disabled = false;
}

runBtn.addEventListener('click', () => {
  if (!isRunning) {
    doRun();
  } else {
    doStop();
  }
});

window.addEventListener('keydown', (e) => {
  if (!runtime) return;
  if (document.activeElement !== canvas) return;
  try {
    console.log("Got key " + e.key);
    executeKey(e.key); // pass string like 'a', 'Enter', etc.
  } catch (err) {
    console.warn('Keyboard runtime error:', err);
  }
});

const resizer = document.getElementById('resizer');
const panes = document.querySelector('.panes');
let isDragging = false;

resizer.addEventListener('mousedown', (e) => {
  isDragging = true;
  document.body.style.cursor = 'col-resize';
});

window.addEventListener('mousemove', (e) => {
  if (!isDragging) return;
  const totalWidth = panes.getBoundingClientRect().width;
  const leftWidth = e.clientX;
  const rightWidth = totalWidth - leftWidth - 4; // 4 = resizer width
  panes.style.gridTemplateColumns = `${leftWidth}px 4px ${rightWidth}px`;
});

window.addEventListener('mouseup', () => {
  isDragging = false;
  document.body.style.cursor = '';
});

document.getElementById("canvas").addEventListener('click', (e) => {
  console.log("Got click!");
  if (!runtime) return;

  const rect = canvas.getBoundingClientRect();
  console.log(e.clientX, rect.left, rect.width);
  const x = (e.clientX - rect.left) / rect.width * 1000;
  const y = (e.clientY - rect.top) /rect.height * 1000;

  try {
    const dpr = window.devicePixelRatio || 1;

  // Match canvas internal size to actual visible size * device pixel ratio
    console.log("Mouse on x: " + x + " Y: " + y);
    executeMouse(x, y);
  } catch (err) {
    console.warn('Mouse runtime error:', err);
  }
});

// // Ctrl/Cmd+Enter
// addEventListener("keydown", (e) => {
//   const isMac = navigator.platform.toLowerCase().includes("mac");
//   if ((isMac ? e.metaKey : e.ctrlKey) && e.key === "Enter") {
//     e.preventDefault();
//     doRun();
//   }
// });

//resizeCanvasToDisplaySize();
editor.focus();

//const observer = new ResizeObserver(() => resizeCanvasToDisplaySize());
//observer.observe(canvas);
