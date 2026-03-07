/**
 * main.js
 *
 * Entry point for the Quanta web IDE.
 *
 * Responsibilities:
 *   - Create and configure the CodeMirror 6 editor with Quanta language support.
 *   - Compile Quanta source via the Rust/WASM `Compiler` on every keystroke
 *     (debounced 1 s) and on explicit Run.
 *   - Drive the canvas runtime (`canvas-runtime.js`) frame-by-frame using the
 *     block-based command stream returned by the WASM runtime.
 *   - Wire up keyboard and mouse events so the running program can react to input.
 *   - Handle file load / save and canvas image export.
 */

// CodeMirror bits (via esm.sh, no local install needed)
//import { EditorView, lineNumbers, highlightActiveLine } from "@codemirror/view";
//import { EditorState } from "@codemirror/state";
//import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
//import { indentOnInput } from "@codemirror/language";
import { oneDark, oneDarkHighlightStyle } from "@codemirror/theme-one-dark";
import {barf, dracula} from 'thememirror';
//import { autocompletion } from "@codemirror/autocomplete";
import {EditorState, RangeSetBuilder, EditorSelection, Compartment} from "@codemirror/state"
import { HighlightStyle, tags as t } from "@codemirror/highlight";

import {
  EditorView, keymap, highlightSpecialChars, drawSelection,
  highlightActiveLine, dropCursor, rectangularSelection,
  crosshairCursor, lineNumbers, highlightActiveLineGutter,
  Decoration, ViewPlugin
} from "@codemirror/view"
import {
  defaultHighlightStyle, syntaxHighlighting, indentOnInput,
  bracketMatching, foldGutter, foldKeymap, indentUnit
} from "@codemirror/language"
import {
  defaultKeymap, history, historyKeymap
} from "@codemirror/commands"
import {
  autocompletion, closeBrackets,
  closeBracketsKeymap, completionKeymap
} from "@codemirror/autocomplete"
import { linter, setDiagnostics } from "@codemirror/lint";
// Language support (your Lezer parser compiled to quanta.js)
import { quanta, quantaSyntax, quantaLanguageSupport } from "./quanta-support.ts";

import { quantaTheme } from "./custom-theme";

// Canvas runtime (drawScript + utilities)
import { drawScript, setup, checkIsCancelled, cancelNow, setIsSafari } from "./canvas-runtime.js";

// WASM glue (wasm-pack output); adjust crate name/path
import initWasm, { Compiler } from "../quanta-lang/pkg/quanta_lang.js";
//import { rustHighlighting } from "../grammar/highlight.js";

const runBtn = document.getElementById("runBtn");

/** The live WASM runtime instance; `undefined` when no program is executing. */
let runtime = undefined;
/** True while a program is running (controls the Run/Stop button state). */
let isRunning = false;

// ---------------------------------------------------------------------------
// Editor configuration
// ---------------------------------------------------------------------------

const fourSpaceIndent = indentUnit.of("    "); // 4 spaces

/**
 * Keymap extension that inserts four spaces on Tab instead of a real tab
 * character, keeping indentation consistent with the language convention.
 */
const insertFourSpaces = keymap.of([{
  key: "Tab",
  run: ({ state, dispatch }) => {
    dispatch(
      state.replaceSelection("    ") // 4 spaces
    );
    return true; // handled
  }
}]);

/** Compartment that allows hot-swapping the font-size theme extension. */
const fontSizeCompartment = new Compartment();

/**
 * Keymap extension that preserves leading indentation on Enter.
 *
 * Extra rules:
 *   - If the current line ends with `{`, the new line gets an extra 4-space indent.
 *   - If the current line ends with `{}` and the cursor is between the braces,
 *     both an indented line and a closing line are inserted.
 */
const newlineSameIndent = keymap.of([{
  key: "Enter",
  run: (view) => {
    const { state } = view;
    const tr = state.changeByRange(range => {
      const line = state.doc.lineAt(range.head);
      let leadingWS = (line.text.match(/^[ \t]*/) || [""])[0]; // copy tabs/spaces exactly
      let extra = "";
      if (line.text.trimEnd().endsWith("{")) {
          if (range.head === line.to) {
          // increase indent after {
          leadingWS += "    "; // add 4 spaces

        }
      }
      if (line.text.trimEnd().endsWith("{}")) {
        if (range.head === line.to - 1) {
        // increase indent after {
          extra += "\n" + leadingWS; // add 4 spaces
          leadingWS += "    "
        }
      }
      const insert = "\n" + leadingWS + extra;
      return {
        changes: { from: range.from, to: range.to, insert },
        range: EditorSelection.cursor(range.from + leadingWS.length + 1)
      };
    });
    view.dispatch(tr, { userEvent: "input" });
    return true;
  }
}]);

// ---------------------------------------------------------------------------
// Diagnostics helpers
// ---------------------------------------------------------------------------

/**
 * Display a compiler/runtime error as a CodeMirror inline diagnostic.
 *
 * @param {import("@codemirror/view").EditorView} editor - The active EditorView.
 * @param {{ start_row: number, start_column: number, end_row: number, end_column: number, get_error_message(): string }} err
 */
export function showError(editor, err) {
  let diagnostics = [];
  const from_line = editor.state.doc.line(Math.max(1, err.start_row));
  const from = Math.min(from_line.to, from_line.from + err.start_column);
  const to_line = editor.state.doc.line(Math.max(1, err.end_row));
  const to = Math.min(to_line.to, to_line.from + err.end_column);
  diagnostics.push({
    from: from,
    to: to, // adjust for token length if needed
    severity: "error",
    message: err.get_error_message()
  });

  editor.dispatch(setDiagnostics(editor.state, diagnostics));
}

/**
 * Log an error to the browser console and show a native alert with
 * the source location and message.
 *
 * @param {{ start_row: number, start_column: number, end_row: number, end_column: number, get_error_message(): string }} err
 */
export function alertError(err) {
    console.log("Error:" + err.get_error_message() + " at "
        + err.start_row + ":" + err.start_column
        + " - " + err.end_row + ":" + err.end_column);
    alert("Error at " + err.start_row + ":" + err.start_column + " - " + err.end_row + ":" + err.end_column + "\n" + err.get_error_message());
}

/**
 * Clear all inline diagnostics from the editor.
 *
 * @param {import("@codemirror/view").EditorView} editor
 */
export function showOk(editor) {
  editor.dispatch(setDiagnostics(editor.state, []));
}

// ---------------------------------------------------------------------------
// Editor initial content
// ---------------------------------------------------------------------------

const STORAGE_KEY = "quanta-editor-code";

const savedCode = localStorage.getItem(STORAGE_KEY);
/** Default program shown when no saved code exists in localStorage. */
const startCode = savedCode || `func mouse(int z, int y) {
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

// ---------------------------------------------------------------------------
// Background compile (on typing)
// ---------------------------------------------------------------------------

/**
 * Compile `src` with a fresh WASM `Compiler` instance and show any error
 * as an inline diagnostic in the editor.
 * Called in the background while the user types; does not start execution.
 *
 * @param {{ view: import("@codemirror/view").EditorView }} editor - EditorView or update object.
 * @param {string} src - Full source text to compile.
 */
async function tryCompile(editor, src) {
  await initWasm();
  let idle_compiler = Compiler.new();
  const compilation_result = await idle_compiler.compile_code(src);   // Rust returns drawing commands (string)
   if (compilation_result.error_code != 0) {
    const err = compilation_result.get_error();
    showError(editor.view, err);
  //   runBtn.disabled = false;
  //   return;
   } else {
  //   showOk(editor);
   }
}

/** Handle for the debounce timer used by `onTyping`. */
let typingTimer = null;

/**
 * CodeMirror update listener that debounces background compilation.
 * On every document change it resets the 1-second timer, then compiles and
 * persists the source to localStorage when the user pauses typing.
 */
const onTyping = EditorView.updateListener.of(update => {
  if (update.docChanged) {
    update.view.dispatch(setDiagnostics(update.state, []));
    clearTimeout(typingTimer);

    // schedule a new one
    typingTimer = setTimeout(() => {
      const code = update.state.doc.toString();

      tryCompile(update, code);
      localStorage.setItem(STORAGE_KEY, editor.state.doc.toString());

    }, 1000); // 1000ms = 1 second pause
  }
});

// ---------------------------------------------------------------------------
// Font-size controls
// ---------------------------------------------------------------------------

/**
 * Build a CodeMirror theme that applies `sizePx` to editor content and gutters.
 *
 * @param {number} sizePx - Font size in pixels.
 * @returns {import("@codemirror/view").Extension}
 */
export function fontSizeTheme(sizePx) {
  return EditorView.theme({
    ".cm-content": { fontSize: sizePx + "px" },
    ".cm-line":    { fontSize: sizePx + "px" },
    ".cm-gutters": { fontSize: sizePx + "px" }
  });
}

// Keep track of current size
let currentFontSize = 18;
let fontSizeExt = fontSizeTheme(currentFontSize);

/**
 * Keymap extension for runtime font-size adjustment:
 *   - `Mod-=`  → increase font size by 1 px.
 *   - `Mod--`  → decrease font size by 1 px (minimum 8 px).
 */
const fontSizeKeys = keymap.of([
  {
    key: "Mod-=",
    run: (view) => {
      currentFontSize += 1;
      view.dispatch({
        effects: fontSizeCompartment.reconfigure(fontSizeTheme(currentFontSize))
      });
      return true;
    }
  },
  {
    key: "Mod--",
    run: (view) => {
      currentFontSize = Math.max(8, currentFontSize - 1);
      view.dispatch({
        effects: fontSizeCompartment.reconfigure(fontSizeTheme(currentFontSize))
      });
      return true;
    }
  }
]);

// ---------------------------------------------------------------------------
// Safari detection
// ---------------------------------------------------------------------------

/** True when the page is running in Safari (detected via user-agent). */
let itIsSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);

setIsSafari(itIsSafari);

// ---------------------------------------------------------------------------
// Editor instantiation
// ---------------------------------------------------------------------------

const editor = new EditorView({
  state: EditorState.create({
    doc: startCode,
     extensions: [
    // A line number gutter
    lineNumbers(),
    // A gutter with code folding markers
     foldGutter(),
    // // Replace non-printable characters with placeholders
     highlightSpecialChars(),
    // // The undo history
     history(),
    // // Replace native cursor/selection with our own
     drawSelection(),
    // // Show a drop cursor when dragging over the editor
    // dropCursor(),
    // // Allow multiple cursors/selections
    // EditorState.allowMultipleSelections.of(true),
    // // Re-indent lines when typing specific input
     indentOnInput(),
    // // Highlight syntax with a default style
    //syntaxHighlighting(rustHighlighting),
    // // Highlight matching brackets near cursor
     bracketMatching(),
    // // Automatically close brackets
     closeBrackets(),
    // // Load the autocompletion system
     autocompletion(),
    // // Allow alt-drag to select rectangular regions
    // rectangularSelection(),
    // // Change the cursor to a crosshair when holding alt
    // crosshairCursor(),
    // // Style the current line specially
     highlightActiveLine(),
    // // Style the gutter for current line specially
     highlightActiveLineGutter(),
    quantaTheme,
    quantaLanguageSupport,
    //keymap.of([{key: "Tab", run: acceptCompletion}]),
    // Highlight text that matches the selected text
    //highlightSelectionMatches(),
    onTyping,
    insertFourSpaces,
    fourSpaceIndent,
    newlineSameIndent,
    fontSizeCompartment.of(fontSizeTheme(currentFontSize)),
    fontSizeKeys,
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
    //
    //   oneDark
    // ]
  }),
  parent: document.getElementById("editor")
});

// ---------------------------------------------------------------------------
// Execution helpers
// ---------------------------------------------------------------------------

/** Clear all inline diagnostics from the global editor instance. */
function clearErrors() {
  editor.dispatch(setDiagnostics(editor.state, []));
}

/**
 * Return a Promise that resolves after `ms` milliseconds.
 * Used to yield control between animation frames in `doRun`.
 *
 * @param {number} ms
 * @returns {Promise<void>}
 */
export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/** Cancel the running program and clear the runtime reference. */
function doStop() {
  (async() => {
    runtime = undefined;
    cancelNow()
  })();
}

/**
 * Invoke the runtime's main entry point asynchronously.
 * The runtime begins producing command blocks that `doRun` will consume.
 */
async function startExecution() {
  let res = runtime.execute();
}

/**
 * Forward a keyboard event key string to the running program's `keyboard` handler.
 *
 * @param {string} key - Key value string (e.g. `"a"`, `"Enter"`, `"ArrowUp"`).
 */
async function executeKey(key) {
  let res = runtime.execute_key(key);
}

/**
 * Forward canvas-relative coordinates to the running program's `mouse` handler.
 *
 * @param {number} x - X position in logical canvas units (0–1000).
 * @param {number} y - Y position in logical canvas units (0–1000).
 */
async function executeMouse(x, y) {
  let res = runtime.execute_mouse(x, y);
}

// window.addEventListener('keydown', (e) => {
//   if (!runtime || !runtime.execute_key) return;

//   try {
//     (async () => {runtime.execute_key(e.key);})(); // pass string like 'a', 'Enter', etc.
//   } catch (err) {
//     console.warn('Keyboard runtime error:', err);
//   }
// });

/**
 * Compile and execute the current editor source.
 *
 * Flow:
 *   1. Reset cancellation state and canvas.
 *   2. Compile with a fresh WASM `Compiler`; show error and abort on failure.
 *   3. Obtain the WASM runtime and call `startExecution()`.
 *   4. Poll `runtime.get_commands()` in a loop, rendering each block via
 *      `drawScript`.  Block status codes:
 *        - `0` → frame complete (composite the buffer to the visible canvas).
 *        - `2` → program ended normally.
 *        - `3` → runtime error (show error, stop loop).
 *   5. Restore idle UI state when done.
 */
function doRun() {
  (async () => {
    try {
      cancelNow(false);
      isRunning = true;
      runBtn.disabled = true;
      setup();
      await initWasm();
      const src = editor.state.doc.toString();
      let compiler = Compiler.new();
      const compilation_result = await compiler.compile_code(src);   // Rust returns drawing commands (string)
      if (compilation_result.error_code != 0) {
        const err = compilation_result.get_error();
        showError(editor, err);
        alertError(err);
        runBtn.disabled = false;
        return;
      } else {
        showOk(editor);
      }
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
          let blockStatus = block.get_status();
          drawScript(commands, blockStatus == 0);
          if (blockStatus == 3) { // Error
            const err = runtime.get_runtime_error();
            showError(editor, err);
            alertError(err);
            need_continue = false;
            break;
           } else if (blockStatus == 2) { // End
            need_continue = false;
            break;
           }
          await sleep(block.sleep_for);

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

// ---------------------------------------------------------------------------
// UI state helpers
// ---------------------------------------------------------------------------

/** Switch the Run button to "Stop" and focus the canvas. */
function setRunningUI() {
  isRunning = true;
  runBtn.textContent = 'Stop';
  runBtn.dataset.state = 'stop';
  runBtn.disabled = false;
  canvas.focus();
}

/** Switch the Run button back to "Run your program!" and mark execution idle. */
function setIdleUI() {
  isRunning = false;
  runBtn.textContent = 'Run your program!';
  runBtn.dataset.state = 'run';
  runBtn.disabled = false;
}

// ---------------------------------------------------------------------------
// Event listeners
// ---------------------------------------------------------------------------

/** Toggle between running and stopping the program when the button is clicked. */
runBtn.addEventListener('click', () => {
  if (!isRunning) {
    doRun();
  } else {
    doStop();
  }
});

/**
 * Forward keyboard events to the running program.
 * Only active when the canvas element has focus, so editor shortcuts
 * are not accidentally captured.
 */
window.addEventListener('keydown', (e) => {
  if (!runtime) return;
  if (document.activeElement !== canvas) return;
  try {
    executeKey(e.key); // pass string like 'a', 'Enter', etc.
  } catch (err) {
    console.warn('Keyboard runtime error:', err);
  }
});

// ---------------------------------------------------------------------------
// Pane resizer (drag handle between editor and canvas)
// ---------------------------------------------------------------------------

const resizer = document.getElementById('resizer');
const panes = document.querySelector('.panes');
let isDragging = false;

resizer.addEventListener('mousedown', (e) => {
  isDragging = true;
  document.body.style.cursor = 'col-resize';
});

/** Update the CSS grid column sizes while the user drags the handle. */
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

/**
 * Forward canvas click coordinates (normalised to 0–1000 logical units)
 * to the running program's `mouse` handler.
 */
document.getElementById("canvas").addEventListener('click', (e) => {
  if (!runtime) return;

  const rect = canvas.getBoundingClientRect();
  const x = (e.clientX - rect.left) / rect.width * 1000;
  const y = (e.clientY - rect.top) /rect.height * 1000;

  try {
    const dpr = window.devicePixelRatio || 1;

  // Match canvas internal size to actual visible size * device pixel ratio
    executeMouse(x, y);
  } catch (err) {
    console.warn('Mouse runtime error:', err);
  }
});

// ---------------------------------------------------------------------------
// File I/O
// ---------------------------------------------------------------------------

/**
 * Trigger a browser download of `text` as a plain-text file named `filename`.
 *
 * @param {string} filename - Suggested download filename.
 * @param {string} text     - File contents.
 */
export function downloadFile(filename, text) {
  const blob = new Blob([text], { type: "text/plain" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

/** Download the current editor source as a `.quanta` file. */
document.getElementById("downloadBtn").addEventListener("click", () => {
  const code = editor.state.doc.toString();

  // Ask user for filename
  let filename = prompt("Enter filename:", "program");
  if (!filename) return; // user pressed Cancel

  // Ensure extension
  if (!filename.endsWith(".quanta")) {
    filename += ".quanta";
  }

  downloadFile(filename, code);
});

// Load file on demand
const fileInput = document.getElementById("fileInput");

/** Open the system file picker to load a `.quanta` source file. */
document.getElementById("loadBtn").addEventListener("click", () => {
  fileInput.value = ""; // reset so selecting the same file again still triggers
  fileInput.click();    // open system file picker
});

/** Export the current canvas contents as a JPEG image. */
document.getElementById("saveBtn").addEventListener("click", () => {
  const canvas = document.getElementById("canvas");
  const image = canvas.toDataURL("image/jpeg", 0.95); // 0.95 is quality

  const filename = prompt("Enter painting name:", "painting");
  if (!filename) return; // user pressed Cancel

  const link = document.createElement("a");
  link.href = image;
  link.download = filename + ".jpg";
  link.click();
});

/** Read the selected file and replace the editor's content with its text. */
fileInput.addEventListener("change", (e) => {
  const file = e.target.files[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = () => {
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: reader.result }
    });
  };
  reader.readAsText(file);
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
