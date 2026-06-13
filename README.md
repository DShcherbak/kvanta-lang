# Kvanta-lang

A graphics-oriented programming language that runs in the browser. Write code, draw shapes, create animations — all in a browser-based IDE with real-time feedback.

The canvas is 1000×1000 virtual pixels and scales responsively to the window size.

## Quick Start

```
# Draw a red circle
setFigureColor(Color::Red);
circle(500, 500, 200);
```

Run the above directly — no functions needed for simple scripts.

## Language Features

### Data Types


| Type          | Example                                 |
| ------------- | --------------------------------------- |
| `int`         | `int x = 42;`                           |
| `float`       | `float ratio = 2.3;`                    |
| `bool`        | `bool on = true;`                       |
| `color`       | `color c = Color::Green;`               |
| `string`      | `string s = "hello";`                   |
| `array<T, N>` | `array<int, 5> nums = {1, 2, 3, 4, 5};` |


Arrays can be nested: `array<array<int, 3>, 3> grid = { {0,1,2}, {3,4,5}, {6,7,8} };`

Expanding array syntax: `array<int, 10> zeros = {0...};`

### Drawing Commands

```
circle(x, y, r)                    -- circle of radius r centered at (x, y)
rectangle(x1, y1, x2, y2)          -- rectangle from (x1, y1) to (x2, y2)
line(x1, y1, x2, y2)               -- line between two points
arc(x, y, r, a1, a2)               -- arc from angle a1 to a2 (degrees, CCW from X axis)
polygon(x1, y1, x2, y2, x3, y3, ..)-- polygon from N >= 3 points

setFigureColor(Color::Red)          -- fill color (default: white)
setLineColor(Color::Blue)           -- stroke color (default: black)
setLineWidth(3)                     -- line width in pixels (default: 1)
```

### Colors

```
Color::Red, Color::Green, Color::Blue, Color::Yellow,
Color::Pink, Color::White, Color::Black, ...
Color::Random                       -- random color
rgb(r, g, b)                        -- custom color from components
Color::Transparent                  -- no fill / transparent
```

### Math Functions

```
abs(x)            -- absolute value
round(x)          -- round to nearest int
ceil(x)           -- round up
floor(x)          -- round down
sqrt(x)           -- square root
decimal(x)        -- cast int to float  (5 / 2 == 2, decimal(5) / 2 == 2.5)
random(a, b)      -- random int in [a, b]
```

### Control Flow

```
if (condition) {
    ...
} else {
    ...
}

for i in (0..10) { ... }    -- inclusive range; decrements if from > to
while (condition) { ... }
```

### Functions

```
func add(int a, int b) -> int {
    return a + b;
}

func draw() {
    circle(500, 500, 100);
}

func main() {
    draw();
}
```

When any function is defined, all top-level code must be inside functions. `main()` is the entry point.

### Global Variables

```
global {
    int score = 0;
    color currentColor = Color::Green;
}
```

Global blocks are accessible from all functions.

### Animation

```
animate()       -- enter animation mode (nothing renders until frame() is called)
frame()         -- push current virtual canvas to screen
sleep(ms)       -- pause execution for ms milliseconds
```

### Event Handlers

Declare these functions to handle input events:

```
func mouse(int x, int y) {
    -- called on canvas click at position (x, y)
}

func keyboard(int key) {
    -- called on keypress when canvas is focused
    if (key == Key::Space) { ... }
    if (key == Key::A) { ... }    -- 'A' key, etc.
}
```

The canvas gains focus on program start or when clicked.

### Example: Interactive Drawing

```
global {
    color c = Color::Blue;
}

func mouse(int x, int y) {
    setFigureColor(c);
    circle(x, y, 20);
}

func keyboard(int key) {
    if (key == Key::Space) {
        c = Color::Random;
    }
}

func main() {
    while (true) {
        
    }
}
```

### Example: Animation

```
func main() {
    animate();
    for i in (0..360) {
        setFigureColor(Color::Red);
        circle(500 + round(decimal(i) * 3.14159 / 180.0 * 200.0), 500, 30);
        frame();
        sleep(16);
    }
}
```

## Installation & Build

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm-pack`: `cargo install wasm-pack`
- [Node.js](https://nodejs.org/) (for Vite frontend)
- [Tree-sitter CLI](https://tree-sitter.github.io/tree-sitter/creating-parsers#installation) (optional, for grammar development)

### Build

```bash
# 1. Clone the repo
git clone <repo-url>
cd kvanta-lang

# 2. Compile Rust → WASM
cd quanta-lang
wasm-pack build --release --target web

# 3. Install frontend dependencies
npm install

# 4. Start dev server
npm run dev
```

Then open [http://localhost:5173](http://localhost:5173).

## Project Structure

```
kvanta-lang/
├── grammar/                 # Parser grammar
│   ├── quanta.grammar       # Lezer grammar (frontend syntax highlighting)
│   ├── grammar.pest         # Pest PEG grammar (backend parser)
│   └── highlight.js         # Syntax highlight rules
│
├── quanta_parser/           # Rust: PEG parser → AST
│   └── src/
│       ├── ast.rs           # AST node types
│       └── ast/builder.rs   # AST construction from parse tree
│
├── quanta-lang/             # Rust: compiler + runtime (compiled to WASM)
│   └── src/
│       ├── compiler.rs      # Compilation pipeline
│       ├── program.rs       # Type checker
│       ├── execution.rs     # Tree-walking interpreter
│       └── runtime.rs       # WASM bindings
│
├── web/                     # Frontend IDE
│   ├── main.js              # Editor setup, WASM integration
│   ├── canvas-runtime.js    # Canvas drawing API
│   └── quanta-support.ts    # CodeMirror language support
│
└── index.html               # Single-page app
```

## Tech Stack

- **Pest** — PEG parser for the backend compiler
- **Lezer** — LR parser for real-time syntax highlighting in the IDE
- **Rust + wasm-pack** — compiler and interpreter compiled to WebAssembly
- **CodeMirror 6** — code editor with syntax highlighting and autocomplete
- **Vite** — frontend build tool and dev server
- **HTML5 Canvas** — rendering target

## Running web runtime tests

Install dependencies first (only needed once):
```
npm install
```

Run all unit tests once:
```
npm test
```

Run in watch mode (re-runs on file save):
```
npm run test:watch
```
