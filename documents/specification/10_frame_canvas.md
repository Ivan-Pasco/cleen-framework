# Frame Canvas Specification (10)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 2.0
**Location:** `/docs/specification/10_frame_canvas.md`

---

## 1. Introduction

`frame.canvas` is the official graphics and animation plugin for Clean Language. It provides a hardware-accelerated 2D rendering API that compiles to WebAssembly, enabling games, visualizations, and interactive graphics applications.

### Key Characteristics

- **Hardware-accelerated rendering** via WebGL/Canvas2D backend
- **Frame-based animation** with automatic delta time calculation
- **Type-safe graphics API** with compile-time validation
- **State-driven architecture** using Clean Language `state:` blocks
- **Plugin-based design** following Frame's standard `blockName:` pattern
- **Zero compiler changes required** - uses standard framework block expansion

### Design Philosophy

- One clear way to accomplish each task
- Follows established Frame plugin patterns
- Explicit bridge function calls over magic syntax
- Predictable performance with direct WASM compilation

---

## 2. Plugin Architecture

### How frame.canvas Works

```
┌─────────────────────────────────────────────────────────────────┐
│                        SOURCE CODE                               │
│                                                                  │
│   import:                                                        │
│       frame.canvas                                               │
│                                                                  │
│   canvasScene: width=800 height=600                             │
│       // drawing code                                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      PLUGIN EXPANSION                            │
│                                                                  │
│   expand_block("canvasScene", attrs, body) returns:             │
│   → integer _canvas_0 = _canvas_init(800, 600)                  │
│   → // drawing code                                             │
│   → _canvas_present(_canvas_0)                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       WASM OUTPUT                                │
│                                                                  │
│   Imports: _canvas_init, _canvas_circle_filled, etc.           │
│   Exports: start, _frame_callback                               │
└─────────────────────────────────────────────────────────────────┘
```

### Plugin Manifest (plugin.toml)

```toml
[plugin]
name = "frame.canvas"
version = "1.0.0"
description = "Canvas rendering plugin for Clean Language"
author = "Clean Language Team"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"
get_keywords = "get_keywords"

[handles]
blocks = ["canvasScene", "draw", "onFrame"]

[paths]
owns = ["src/canvas"]
auto_create = true
patterns = ["*.cln"]
implicit_import = true

[bridge]
functions = [
  # Canvas lifecycle
  { name = "_canvas_init", params = ["integer", "integer"], returns = "integer" },
  { name = "_canvas_clear", params = ["integer"], returns = "integer" },
  { name = "_canvas_clear_color", params = ["integer", "string"], returns = "integer" },
  { name = "_canvas_present", params = ["integer"], returns = "integer" },

  # Shapes
  { name = "_canvas_circle", params = ["integer", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_circle_filled", params = ["integer", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_rect", params = ["integer", "number", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_rect_filled", params = ["integer", "number", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_line", params = ["integer", "number", "number", "number", "number", "number", "string"], returns = "integer" },

  # Text
  { name = "_canvas_text", params = ["integer", "string", "number", "number", "number", "string"], returns = "integer" },

  # Images
  { name = "_canvas_image", params = ["integer", "string", "number", "number", "number", "number"], returns = "integer" },

  # Transforms
  { name = "_canvas_save", params = ["integer"], returns = "integer" },
  { name = "_canvas_restore", params = ["integer"], returns = "integer" },
  { name = "_canvas_translate", params = ["integer", "number", "number"], returns = "integer" },
  { name = "_canvas_rotate", params = ["integer", "number"], returns = "integer" },
  { name = "_canvas_scale", params = ["integer", "number", "number"], returns = "integer" },

  # Animation
  { name = "_canvas_request_frame", params = ["integer"], returns = "integer" },
  { name = "_canvas_cancel_frame", params = ["integer"], returns = "integer" },
  { name = "_canvas_get_delta_time", params = [], returns = "number" },

  # Input
  { name = "_canvas_pointer_x", params = [], returns = "number" },
  { name = "_canvas_pointer_y", params = [], returns = "number" },
  { name = "_input_key_down", params = ["string"], returns = "boolean" },
]
```

---

## 3. Canvas Scene Declaration

### Syntax

Canvas scenes use the standard framework block syntax with `canvasScene:`:

```clean
import:
    frame.canvas

canvasScene: width=800 height=600 background="#1a1a2e"
    // Canvas body - expanded by plugin
```

### Configuration Attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `width` | integer | 800 | Canvas width in pixels |
| `height` | integer | 600 | Canvas height in pixels |
| `background` | string | "#000000" | Background color (hex) |
| `id` | string | "_canvas_0" | Canvas identifier for multiple canvases |

### Examples

**Basic Canvas**:
```clean
import:
    frame.canvas

canvasScene: width=1024 height=768 background="#2c3e50"
    _canvas_circle_filled(_canvas_0, 400.0, 300.0, 50.0, "#e74c3c")
```

**Multiple Canvases**:
```clean
import:
    frame.canvas

canvasScene: id="game" width=800 height=600 background="#1a1a2e"
    _canvas_circle_filled(_canvas_game, 400.0, 300.0, 50.0, "#3498db")

canvasScene: id="minimap" width=200 height=150 background="#2c3e50"
    _canvas_rect_filled(_canvas_minimap, 0.0, 0.0, 200.0, 150.0, "#34495e")
```

### Plugin Expansion

The plugin expands `canvasScene:` to:

```clean
// Input:
canvasScene: width=800 height=600 background="#1a1a2e"
    _canvas_circle_filled(_canvas_0, 400.0, 300.0, 50.0, "#ff6b6b")

// Output (generated by plugin):
integer _canvas_0 = _canvas_init(800, 600)
_canvas_clear_color(_canvas_0, "#1a1a2e")
_canvas_circle_filled(_canvas_0, 400.0, 300.0, 50.0, "#ff6b6b")
_canvas_present(_canvas_0)
```

---

## 4. Drawing Primitives

All drawing uses direct bridge function calls. This ensures predictable behavior and optimal WASM compilation.

### Circle

```clean
// Filled circle
_canvas_circle_filled(canvasId, x, y, radius, color)

// Stroked circle (outline only)
_canvas_circle(canvasId, x, y, radius, color)
```

**Parameters**:
- `canvasId: integer` - Canvas identifier
- `x: number` - Center X coordinate
- `y: number` - Center Y coordinate
- `radius: number` - Circle radius
- `color: string` - Fill/stroke color (hex, rgb, rgba)

**Example**:
```clean
_canvas_circle_filled(_canvas_0, 400.0, 300.0, 50.0, "#e74c3c")
_canvas_circle(_canvas_0, 400.0, 300.0, 60.0, "#ffffff")
```

### Rectangle

```clean
// Filled rectangle
_canvas_rect_filled(canvasId, x, y, width, height, color)

// Stroked rectangle (outline only)
_canvas_rect(canvasId, x, y, width, height, color)
```

**Parameters**:
- `canvasId: integer` - Canvas identifier
- `x: number` - Top-left X coordinate
- `y: number` - Top-left Y coordinate
- `width: number` - Rectangle width
- `height: number` - Rectangle height
- `color: string` - Fill/stroke color

**Example**:
```clean
_canvas_rect_filled(_canvas_0, 100.0, 100.0, 200.0, 150.0, "#3498db")
_canvas_rect(_canvas_0, 100.0, 100.0, 200.0, 150.0, "#ffffff")
```

### Line

```clean
_canvas_line(canvasId, x1, y1, x2, y2, strokeWidth, color)
```

**Parameters**:
- `canvasId: integer` - Canvas identifier
- `x1, y1: number` - Start point
- `x2, y2: number` - End point
- `strokeWidth: number` - Line thickness
- `color: string` - Line color

**Example**:
```clean
_canvas_line(_canvas_0, 0.0, 0.0, 800.0, 600.0, 2.0, "#ecf0f1")
```

### Text

```clean
_canvas_text(canvasId, text, x, y, fontSize, color)
```

**Parameters**:
- `canvasId: integer` - Canvas identifier
- `text: string` - Text to render
- `x: number` - Left position
- `y: number` - Baseline position
- `fontSize: number` - Font size in pixels
- `color: string` - Text color

**Example**:
```clean
_canvas_text(_canvas_0, "Score: 1000", 20.0, 30.0, 24.0, "#ffffff")
```

### Image

```clean
_canvas_image(canvasId, src, x, y, width, height)
```

**Parameters**:
- `canvasId: integer` - Canvas identifier
- `src: string` - Image path or URL
- `x: number` - Destination X
- `y: number` - Destination Y
- `width: number` - Display width
- `height: number` - Display height

**Example**:
```clean
_canvas_image(_canvas_0, "sprites/player.png", 100.0, 200.0, 64.0, 64.0)
```

---

## 5. Canvas Operations

### Clear

```clean
// Clear with default background
_canvas_clear(canvasId)

// Clear with specific color
_canvas_clear_color(canvasId, color)
```

**Example**:
```clean
_canvas_clear_color(_canvas_0, "#1a1a2e")
```

### Present

```clean
_canvas_present(canvasId)
```

Flushes the drawing buffer to screen. Called automatically at end of frame in animation loops.

---

## 6. Transforms

Transforms modify the coordinate system for subsequent drawing operations.

### Save/Restore

```clean
_canvas_save(canvasId)      // Push current state
_canvas_restore(canvasId)   // Pop and restore state
```

### Translate

```clean
_canvas_translate(canvasId, x, y)
```

Move the origin point.

### Rotate

```clean
_canvas_rotate(canvasId, angleDegrees)
```

Rotate around current origin.

### Scale

```clean
_canvas_scale(canvasId, scaleX, scaleY)
```

Scale the coordinate system.

### Example: Rotating Rectangle

```clean
functions:
    void drawRotatedRect(integer canvas, number x, number y, number angle)
        _canvas_save(canvas)
        _canvas_translate(canvas, x, y)
        _canvas_rotate(canvas, angle)
        _canvas_rect_filled(canvas, -25.0, -25.0, 50.0, 50.0, "#e74c3c")
        _canvas_restore(canvas)
```

---

## 7. Animation

Animation uses a convention-based callback pattern. The runtime calls `_frame_callback` every frame.

### Animation Setup

```clean
import:
    frame.canvas

state:
    number ballX = 400.0
    number ballY = 300.0
    number velocityX = 200.0
    number velocityY = 150.0

// Called by runtime every frame
functions:
    void _frame_callback(integer canvasId)
        // Get delta time
        number dt = _canvas_get_delta_time()

        // Update physics
        ballX = ballX + velocityX * dt
        ballY = ballY + velocityY * dt

        // Bounce off walls
        if ballX < 20.0 or ballX > 780.0
            velocityX = velocityX * -1.0
        if ballY < 20.0 or ballY > 580.0
            velocityY = velocityY * -1.0

        // Render
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_circle_filled(canvasId, ballX, ballY, 20.0, "#e74c3c")

start()
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)
```

### Animation Bridge Functions

```clean
// Request animation loop (runtime will call _frame_callback)
integer _canvas_request_frame(integer canvasId)

// Stop animation
integer _canvas_cancel_frame(integer frameId)

// Get time since last frame in seconds
number _canvas_get_delta_time()
```

### Frame-Independent Movement

Always multiply velocities by delta time for consistent movement:

```clean
// CORRECT - frame-independent
ballX = ballX + velocity * deltaTime

// WRONG - varies with frame rate
ballX = ballX + velocity
```

---

## 8. State Management

Canvas applications use Clean Language's standard `state:` blocks for persistent state.

### Basic State

```clean
import:
    frame.canvas

state:
    number playerX = 100.0
    number playerY = 300.0
    integer score = 0
    boolean gameOver = false
```

### State with Watch

```clean
state:
    integer health = 100

watch health:
    if health <= 0
        gameOver = true
        print("Game Over!")
```

### Complex State

```clean
class Player
    number x
    number y
    number speed

    constructor(number startX, number startY)
        x = startX
        y = startY
        speed = 200.0

    functions:
        void move(number dx, number dy, number dt)
            x = x + dx * speed * dt
            y = y + dy * speed * dt

state:
    Player player = Player(400.0, 300.0)
    list<Enemy> enemies = []
```

---

## 9. Input Handling

Input is queried via bridge functions. The runtime tracks input state.

### Pointer/Mouse

```clean
// Get current pointer position
number mouseX = _canvas_pointer_x()
number mouseY = _canvas_pointer_y()
```

### Keyboard

```clean
// Check if key is currently pressed
boolean isMovingLeft = _input_key_down("ArrowLeft")
boolean isMovingRight = _input_key_down("ArrowRight")
boolean isJumping = _input_key_down("Space")
```

### Input Example

```clean
functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        // Handle input
        if _input_key_down("ArrowLeft")
            playerX = playerX - 200.0 * dt
        if _input_key_down("ArrowRight")
            playerX = playerX + 200.0 * dt
        if _input_key_down("ArrowUp")
            playerY = playerY - 200.0 * dt
        if _input_key_down("ArrowDown")
            playerY = playerY + 200.0 * dt

        // Render
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_circle_filled(canvasId, playerX, playerY, 20.0, "#3498db")
```

---

## 10. Complete Examples

### Bouncing Ball

```clean
import:
    frame.canvas

state:
    number ballX = 400.0
    number ballY = 100.0
    number velocityX = 250.0
    number velocityY = 0.0
    number gravity = 800.0
    number bounceDamp = 0.85
    number groundY = 560.0

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        // Apply gravity
        velocityY = velocityY + gravity * dt

        // Update position
        ballX = ballX + velocityX * dt
        ballY = ballY + velocityY * dt

        // Bounce off ground
        if ballY > groundY
            ballY = groundY
            velocityY = velocityY * -1.0 * bounceDamp

        // Bounce off walls
        if ballX < 20.0
            ballX = 20.0
            velocityX = velocityX * -1.0
        if ballX > 780.0
            ballX = 780.0
            velocityX = velocityX * -1.0

        // Render
        _canvas_clear_color(canvasId, "#0f0f23")
        _canvas_circle_filled(canvasId, ballX, ballY, 20.0, "#ff6b6b")

        // Draw ground line
        _canvas_line(canvasId, 0.0, 580.0, 800.0, 580.0, 2.0, "#4a4a6a")

start()
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)
```

### Interactive Drawing

```clean
import:
    frame.canvas

state:
    list<number> pointsX = []
    list<number> pointsY = []
    boolean isDrawing = false

functions:
    void _frame_callback(integer canvasId)
        // Get mouse position
        number mouseX = _canvas_pointer_x()
        number mouseY = _canvas_pointer_y()

        // Check if mouse button is down (simplified - actual impl needs bridge function)
        if isDrawing
            pointsX.push(mouseX)
            pointsY.push(mouseY)

        // Render
        _canvas_clear_color(canvasId, "#ffffff")

        // Draw all points
        integer i = 0
        while i < pointsX.length()
            _canvas_circle_filled(canvasId, pointsX.get(i), pointsY.get(i), 5.0, "#333333")
            i = i + 1

        // Draw cursor
        _canvas_circle(canvasId, mouseX, mouseY, 10.0, "#999999")

start()
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)
```

### Simple Game: Paddle

```clean
import:
    frame.canvas

state:
    // Ball
    number ballX = 400.0
    number ballY = 300.0
    number ballVelX = 300.0
    number ballVelY = 200.0

    // Paddle
    number paddleX = 350.0
    number paddleY = 550.0
    number paddleWidth = 100.0
    number paddleHeight = 15.0
    number paddleSpeed = 400.0

    // Game
    integer score = 0
    boolean gameOver = false

functions:
    void _frame_callback(integer canvasId)
        if gameOver
            renderGameOver(canvasId)
            return

        number dt = _canvas_get_delta_time()

        // Move paddle
        if _input_key_down("ArrowLeft")
            paddleX = paddleX - paddleSpeed * dt
        if _input_key_down("ArrowRight")
            paddleX = paddleX + paddleSpeed * dt

        // Clamp paddle to screen
        if paddleX < 0.0
            paddleX = 0.0
        if paddleX > 800.0 - paddleWidth
            paddleX = 800.0 - paddleWidth

        // Move ball
        ballX = ballX + ballVelX * dt
        ballY = ballY + ballVelY * dt

        // Bounce off walls
        if ballX < 10.0 or ballX > 790.0
            ballVelX = ballVelX * -1.0

        // Bounce off top
        if ballY < 10.0
            ballVelY = ballVelY * -1.0

        // Check paddle collision
        if ballY > paddleY - 10.0 and ballY < paddleY + paddleHeight
            if ballX > paddleX and ballX < paddleX + paddleWidth
                ballVelY = ballVelY * -1.0
                score = score + 10

        // Check game over
        if ballY > 600.0
            gameOver = true

        // Render
        render(canvasId)

    void render(integer canvasId)
        _canvas_clear_color(canvasId, "#1a1a2e")

        // Draw ball
        _canvas_circle_filled(canvasId, ballX, ballY, 10.0, "#e74c3c")

        // Draw paddle
        _canvas_rect_filled(canvasId, paddleX, paddleY, paddleWidth, paddleHeight, "#3498db")

        // Draw score
        _canvas_text(canvasId, "Score: " + score.toString(), 20.0, 30.0, 24.0, "#ffffff")

    void renderGameOver(integer canvasId)
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_text(canvasId, "GAME OVER", 300.0, 280.0, 48.0, "#e74c3c")
        _canvas_text(canvasId, "Final Score: " + score.toString(), 320.0, 340.0, 24.0, "#ffffff")
        _canvas_text(canvasId, "Press R to restart", 310.0, 400.0, 18.0, "#888888")

start()
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)
```

---

## 11. Folder Conventions

Files in `src/canvas/` automatically import `frame.canvas`:

```
myapp/
├── frame.toml
├── src/
│   ├── canvas/              # Owned by frame.canvas
│   │   ├── GameScene.cln    # Implicitly imports frame.canvas
│   │   ├── MenuScene.cln
│   │   └── utils.cln
│   └── main.cln
```

**GameScene.cln** (no import needed):
```clean
// src/canvas/GameScene.cln
// frame.canvas is implicitly imported

state:
    number x = 0.0

functions:
    void _frame_callback(integer canvasId)
        x = x + 100.0 * _canvas_get_delta_time()
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_circle_filled(canvasId, x, 300.0, 20.0, "#ff6b6b")

start()
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)
```

---

## 12. DOM Overlay Integration

For standard UI elements (buttons, forms, text inputs), use **frame.ui** components as a DOM overlay on top of the canvas.

### Architecture

```
┌─────────────────────────────────────┐
│         DOM Layer (frame.ui)        │  ← Buttons, forms, HUD
│  <div class="game-ui">              │
│    <button>Pause</button>           │
│    <div class="score">1000</div>    │
│  </div>                             │
├─────────────────────────────────────┤
│       Canvas Layer (frame.canvas)   │  ← Game graphics
│  ┌─────────────────────────────────┐│
│  │  ●  ▲  ■                        ││
│  │       game world                ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### Example: Game with HUD

```html
<!-- pages/game.html -->
<!DOCTYPE html>
<html>
<head>
    <title>My Game</title>
    <style>
        .game-container { position: relative; }
        .game-hud {
            position: absolute;
            top: 10px;
            left: 10px;
            color: white;
            font-size: 24px;
        }
        .pause-btn {
            position: absolute;
            top: 10px;
            right: 10px;
        }
    </style>
</head>
<body>
    <div class="game-container">
        <canvas id="game-canvas" width="800" height="600"></canvas>

        <!-- DOM overlay for UI -->
        <div class="game-hud">
            Score: {{score}}
        </div>
        <button class="pause-btn" onclick="togglePause">
            {{paused ? "Resume" : "Pause"}}
        </button>
    </div>
</body>
</html>
```

---

## 13. Bridge Function Reference

### Canvas Lifecycle

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_canvas_init` | `width: integer, height: integer` | `integer` | Initialize canvas, returns ID |
| `_canvas_clear` | `canvasId: integer` | `integer` | Clear canvas |
| `_canvas_clear_color` | `canvasId: integer, color: string` | `integer` | Clear with color |
| `_canvas_present` | `canvasId: integer` | `integer` | Flush to screen |

### Drawing

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_canvas_circle` | `canvasId, x, y, radius, color` | `integer` | Draw circle outline |
| `_canvas_circle_filled` | `canvasId, x, y, radius, color` | `integer` | Draw filled circle |
| `_canvas_rect` | `canvasId, x, y, width, height, color` | `integer` | Draw rectangle outline |
| `_canvas_rect_filled` | `canvasId, x, y, width, height, color` | `integer` | Draw filled rectangle |
| `_canvas_line` | `canvasId, x1, y1, x2, y2, strokeWidth, color` | `integer` | Draw line |
| `_canvas_text` | `canvasId, text, x, y, fontSize, color` | `integer` | Draw text |
| `_canvas_image` | `canvasId, src, x, y, width, height` | `integer` | Draw image |

### Transforms

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_canvas_save` | `canvasId: integer` | `integer` | Save state |
| `_canvas_restore` | `canvasId: integer` | `integer` | Restore state |
| `_canvas_translate` | `canvasId, x, y` | `integer` | Move origin |
| `_canvas_rotate` | `canvasId, angleDegrees` | `integer` | Rotate |
| `_canvas_scale` | `canvasId, scaleX, scaleY` | `integer` | Scale |

### Animation

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_canvas_request_frame` | `canvasId: integer` | `integer` | Start animation loop |
| `_canvas_cancel_frame` | `frameId: integer` | `integer` | Stop animation |
| `_canvas_get_delta_time` | none | `number` | Get frame delta in seconds |

### Input

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_canvas_pointer_x` | none | `number` | Get mouse/touch X |
| `_canvas_pointer_y` | none | `number` | Get mouse/touch Y |
| `_input_key_down` | `key: string` | `boolean` | Check if key is pressed |

---

## 14. Runtime Implementation

The JavaScript runtime provides bridge function implementations:

```javascript
// canvas-bridge.js (simplified)
const canvases = new Map();
let lastFrameTime = performance.now();

function _canvas_init(width, height) {
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    document.body.appendChild(canvas);

    const id = canvases.size;
    canvases.set(id, {
        canvas,
        ctx: canvas.getContext('2d'),
        wantsAnimation: false
    });
    return id;
}

function _canvas_circle_filled(canvasId, x, y, radius, color) {
    const { ctx } = canvases.get(canvasId);
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fill();
    return 0;
}

function _canvas_request_frame(canvasId) {
    const canvasData = canvases.get(canvasId);
    canvasData.wantsAnimation = true;

    function animate() {
        const now = performance.now();
        lastFrameTime = now;

        // Call WASM _frame_callback
        if (instance.exports._frame_callback) {
            instance.exports._frame_callback(canvasId);
        }

        if (canvasData.wantsAnimation) {
            requestAnimationFrame(animate);
        }
    }

    requestAnimationFrame(animate);
    return 0;
}

function _canvas_get_delta_time() {
    const now = performance.now();
    const dt = (now - lastFrameTime) / 1000;
    return Math.min(dt, 0.1); // Cap at 100ms
}
```

---

## 15. Audio System

The audio system provides sound effects and music playback for games.

### Loading Audio

```clean
// Load a sound effect (short, can play multiple times)
integer soundId = _audio_load_sound("sounds/jump.wav")

// Load music (long, streaming, one at a time)
integer musicId = _audio_load_music("music/background.mp3")
```

### Playing Sounds

```clean
// Play sound effect (returns instance ID for control)
integer instanceId = _audio_play(soundId, volume, loop)

// Parameters:
// - soundId: integer - ID from _audio_load_sound
// - volume: number - 0.0 to 1.0
// - loop: boolean - true for looping

// Example
integer jumpSound = _audio_load_sound("sounds/jump.wav")
_audio_play(jumpSound, 0.8, false)
```

### Music Playback

```clean
// Play music (replaces any currently playing music)
_audio_play_music(musicId, volume, loop)

// Pause/resume music
_audio_pause_music()
_audio_resume_music()

// Stop music
_audio_stop_music()

// Set music volume (0.0 to 1.0)
_audio_set_music_volume(volume)
```

### Sound Control

```clean
// Stop a specific sound instance
_audio_stop(instanceId)

// Stop all sounds
_audio_stop_all()

// Set master volume (affects all audio)
_audio_set_master_volume(volume)

// Check if sound is playing
boolean playing = _audio_is_playing(instanceId)
```

### Spatial Audio (2D Panning)

```clean
// Play sound with position (for positional audio)
integer instanceId = _audio_play_at(soundId, x, y, volume)

// Set listener position (usually the player/camera)
_audio_set_listener(x, y)
```

### Example: Game Audio

```clean
import:
    frame.canvas

state:
    integer jumpSound = 0
    integer coinSound = 0
    integer hurtSound = 0
    integer bgMusic = 0

start()
    // Load all audio assets
    jumpSound = _audio_load_sound("sounds/jump.wav")
    coinSound = _audio_load_sound("sounds/coin.wav")
    hurtSound = _audio_load_sound("sounds/hurt.wav")
    bgMusic = _audio_load_music("music/level1.mp3")

    // Start background music
    _audio_play_music(bgMusic, 0.5, true)

    // Initialize canvas
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)

functions:
    void onJump()
        _audio_play(jumpSound, 0.8, false)

    void onCoinCollect()
        _audio_play(coinSound, 1.0, false)

    void onPlayerHurt()
        _audio_play(hurtSound, 1.0, false)
        // Brief music duck
        _audio_set_music_volume(0.2)
```

### Audio Bridge Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_audio_load_sound` | `path: string` | `integer` | Load sound effect, returns ID |
| `_audio_load_music` | `path: string` | `integer` | Load music track, returns ID |
| `_audio_play` | `soundId, volume, loop` | `integer` | Play sound, returns instance ID |
| `_audio_play_at` | `soundId, x, y, volume` | `integer` | Play with position |
| `_audio_stop` | `instanceId: integer` | `integer` | Stop sound instance |
| `_audio_stop_all` | none | `integer` | Stop all sounds |
| `_audio_play_music` | `musicId, volume, loop` | `integer` | Play music |
| `_audio_pause_music` | none | `integer` | Pause music |
| `_audio_resume_music` | none | `integer` | Resume music |
| `_audio_stop_music` | none | `integer` | Stop music |
| `_audio_set_music_volume` | `volume: number` | `integer` | Set music volume |
| `_audio_set_master_volume` | `volume: number` | `integer` | Set master volume |
| `_audio_is_playing` | `instanceId: integer` | `boolean` | Check if playing |
| `_audio_set_listener` | `x, y: number` | `integer` | Set listener position |

---

## 16. Sprite & Animation System

Sprites enable efficient rendering of game characters and objects with animation support.

### Loading Sprites

```clean
// Load a single sprite image
integer spriteId = _sprite_load("sprites/player.png")

// Load a sprite sheet (grid-based)
integer sheetId = _sprite_load_sheet("sprites/player_sheet.png", frameWidth, frameHeight)

// Parameters:
// - path: string - Image path
// - frameWidth: integer - Width of each frame in pixels
// - frameHeight: integer - Height of each frame in pixels
```

### Drawing Sprites

```clean
// Draw sprite at position
_sprite_draw(canvasId, spriteId, x, y)

// Draw sprite with size
_sprite_draw_sized(canvasId, spriteId, x, y, width, height)

// Draw sprite with rotation and scale
_sprite_draw_ex(canvasId, spriteId, x, y, rotation, scaleX, scaleY, anchorX, anchorY)

// Parameters:
// - anchorX, anchorY: 0.0 to 1.0 (0.5, 0.5 = center)
```

### Sprite Sheet Frames

```clean
// Draw specific frame from sprite sheet
_sprite_draw_frame(canvasId, sheetId, frameIndex, x, y)

// Draw frame with size
_sprite_draw_frame_sized(canvasId, sheetId, frameIndex, x, y, width, height)

// Get frame count
integer frameCount = _sprite_get_frame_count(sheetId)
```

### Animation Helper

For frame-based animation, manage the frame index in state:

```clean
state:
    integer playerSheet = 0
    integer currentFrame = 0
    number animTimer = 0.0
    number frameTime = 0.1  // 10 FPS animation

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        // Update animation
        animTimer = animTimer + dt
        if animTimer >= frameTime
            animTimer = animTimer - frameTime
            currentFrame = (currentFrame + 1) % _sprite_get_frame_count(playerSheet)

        // Draw
        _canvas_clear_color(canvasId, "#1a1a2e")
        _sprite_draw_frame(canvasId, playerSheet, currentFrame, playerX, playerY)
```

### Animation Class Helper

```clean
class Animation
    integer sheetId
    integer startFrame
    integer endFrame
    number frameTime
    boolean loop

    integer currentFrame
    number timer
    boolean finished

    constructor(integer sheet, integer start, integer end, number fps, boolean shouldLoop)
        sheetId = sheet
        startFrame = start
        endFrame = end
        frameTime = 1.0 / fps
        loop = shouldLoop
        currentFrame = start
        timer = 0.0
        finished = false

    functions:
        void update(number dt)
            if finished and not loop
                return

            timer = timer + dt
            if timer >= frameTime
                timer = timer - frameTime
                currentFrame = currentFrame + 1

                if currentFrame > endFrame
                    if loop
                        currentFrame = startFrame
                    else
                        currentFrame = endFrame
                        finished = true

        void draw(integer canvasId, number x, number y)
            _sprite_draw_frame(canvasId, sheetId, currentFrame, x, y)

        void reset()
            currentFrame = startFrame
            timer = 0.0
            finished = false
```

### Example: Animated Character

```clean
import:
    frame.canvas

state:
    integer playerSheet = 0
    Animation walkAnim = null
    Animation idleAnim = null
    Animation currentAnim = null
    number playerX = 400.0
    number playerY = 300.0
    boolean facingRight = true

start()
    playerSheet = _sprite_load_sheet("sprites/player.png", 32, 32)

    // Create animations (sheet, startFrame, endFrame, fps, loop)
    idleAnim = Animation(playerSheet, 0, 3, 8.0, true)
    walkAnim = Animation(playerSheet, 4, 11, 12.0, true)
    currentAnim = idleAnim

    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        // Handle input
        boolean moving = false
        if _input_key_down("ArrowLeft")
            playerX = playerX - 200.0 * dt
            facingRight = false
            moving = true
        if _input_key_down("ArrowRight")
            playerX = playerX + 200.0 * dt
            facingRight = true
            moving = true

        // Switch animation
        if moving and currentAnim != walkAnim
            currentAnim = walkAnim
            currentAnim.reset()
        if not moving and currentAnim != idleAnim
            currentAnim = idleAnim
            currentAnim.reset()

        // Update animation
        currentAnim.update(dt)

        // Draw
        _canvas_clear_color(canvasId, "#1a1a2e")

        if facingRight
            currentAnim.draw(canvasId, playerX, playerY)
        else
            // Flip horizontally
            _canvas_save(canvasId)
            _canvas_translate(canvasId, playerX + 16.0, playerY)
            _canvas_scale(canvasId, -1.0, 1.0)
            _canvas_translate(canvasId, -16.0, 0.0)
            _sprite_draw_frame(canvasId, playerSheet, currentAnim.currentFrame, 0.0, 0.0)
            _canvas_restore(canvasId)
```

### Sprite Bridge Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_sprite_load` | `path: string` | `integer` | Load sprite image |
| `_sprite_load_sheet` | `path, frameW, frameH` | `integer` | Load sprite sheet |
| `_sprite_draw` | `canvasId, spriteId, x, y` | `integer` | Draw sprite |
| `_sprite_draw_sized` | `canvasId, spriteId, x, y, w, h` | `integer` | Draw with size |
| `_sprite_draw_ex` | `canvasId, spriteId, x, y, rot, sx, sy, ax, ay` | `integer` | Draw with transform |
| `_sprite_draw_frame` | `canvasId, sheetId, frame, x, y` | `integer` | Draw sheet frame |
| `_sprite_draw_frame_sized` | `canvasId, sheetId, frame, x, y, w, h` | `integer` | Draw frame with size |
| `_sprite_get_frame_count` | `sheetId: integer` | `integer` | Get total frames |
| `_sprite_get_width` | `spriteId: integer` | `integer` | Get sprite width |
| `_sprite_get_height` | `spriteId: integer` | `integer` | Get sprite height |

---

## 17. Complete Input System

### Mouse/Pointer Input

```clean
// Position
number mouseX = _input_mouse_x()
number mouseY = _input_mouse_y()

// Button state (0=left, 1=middle, 2=right)
boolean leftDown = _input_mouse_down(0)
boolean rightDown = _input_mouse_down(2)

// Button just pressed this frame
boolean leftPressed = _input_mouse_pressed(0)
boolean leftReleased = _input_mouse_released(0)

// Scroll wheel
number scrollX = _input_scroll_x()
number scrollY = _input_scroll_y()
```

### Keyboard Input

```clean
// Key held down
boolean isDown = _input_key_down("ArrowLeft")

// Key just pressed this frame
boolean justPressed = _input_key_pressed("Space")

// Key just released this frame
boolean justReleased = _input_key_released("Space")

// Get last typed character (for text input)
string typed = _input_get_typed()
```

### Key Names

Standard key names (matching JavaScript KeyboardEvent.code):

| Category | Keys |
|----------|------|
| Arrows | `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight` |
| Letters | `KeyA` through `KeyZ` |
| Numbers | `Digit0` through `Digit9` |
| Function | `F1` through `F12` |
| Modifiers | `ShiftLeft`, `ShiftRight`, `ControlLeft`, `ControlRight`, `AltLeft`, `AltRight` |
| Special | `Space`, `Enter`, `Escape`, `Tab`, `Backspace`, `Delete` |
| Numpad | `Numpad0` through `Numpad9`, `NumpadAdd`, `NumpadSubtract` |

### Touch Input

```clean
// Number of active touches
integer touchCount = _input_touch_count()

// Get touch position by index
number touchX = _input_touch_x(index)
number touchY = _input_touch_y(index)

// Get touch ID (for tracking)
integer touchId = _input_touch_id(index)

// Check if touch just started/ended
boolean touchStarted = _input_touch_started(index)
boolean touchEnded = _input_touch_ended(index)
```

### Gamepad Input

```clean
// Check if gamepad is connected
boolean connected = _input_gamepad_connected(gamepadIndex)

// Button state (standard mapping)
boolean aButton = _input_gamepad_button(gamepadIndex, 0)   // A / Cross
boolean bButton = _input_gamepad_button(gamepadIndex, 1)   // B / Circle
boolean xButton = _input_gamepad_button(gamepadIndex, 2)   // X / Square
boolean yButton = _input_gamepad_button(gamepadIndex, 3)   // Y / Triangle
boolean lb = _input_gamepad_button(gamepadIndex, 4)        // Left bumper
boolean rb = _input_gamepad_button(gamepadIndex, 5)        // Right bumper
boolean lt = _input_gamepad_button(gamepadIndex, 6)        // Left trigger
boolean rt = _input_gamepad_button(gamepadIndex, 7)        // Right trigger
boolean select = _input_gamepad_button(gamepadIndex, 8)    // Select/Back
boolean start = _input_gamepad_button(gamepadIndex, 9)     // Start
boolean l3 = _input_gamepad_button(gamepadIndex, 10)       // Left stick click
boolean r3 = _input_gamepad_button(gamepadIndex, 11)       // Right stick click
boolean dpadUp = _input_gamepad_button(gamepadIndex, 12)   // D-pad up
boolean dpadDown = _input_gamepad_button(gamepadIndex, 13) // D-pad down
boolean dpadLeft = _input_gamepad_button(gamepadIndex, 14) // D-pad left
boolean dpadRight = _input_gamepad_button(gamepadIndex, 15) // D-pad right

// Analog sticks (-1.0 to 1.0)
number leftX = _input_gamepad_axis(gamepadIndex, 0)   // Left stick X
number leftY = _input_gamepad_axis(gamepadIndex, 1)   // Left stick Y
number rightX = _input_gamepad_axis(gamepadIndex, 2)  // Right stick X
number rightY = _input_gamepad_axis(gamepadIndex, 3)  // Right stick Y

// Vibration/rumble
_input_gamepad_vibrate(gamepadIndex, duration, weakMagnitude, strongMagnitude)
```

### Input Example: Comprehensive Controls

```clean
state:
    number playerX = 400.0
    number playerY = 300.0
    number speed = 200.0

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()
        number dx = 0.0
        number dy = 0.0

        // Keyboard
        if _input_key_down("ArrowLeft") or _input_key_down("KeyA")
            dx = dx - 1.0
        if _input_key_down("ArrowRight") or _input_key_down("KeyD")
            dx = dx + 1.0
        if _input_key_down("ArrowUp") or _input_key_down("KeyW")
            dy = dy - 1.0
        if _input_key_down("ArrowDown") or _input_key_down("KeyS")
            dy = dy + 1.0

        // Gamepad (if connected)
        if _input_gamepad_connected(0)
            number gx = _input_gamepad_axis(0, 0)
            number gy = _input_gamepad_axis(0, 1)

            // Apply deadzone
            if gx > 0.2 or gx < -0.2
                dx = dx + gx
            if gy > 0.2 or gy < -0.2
                dy = dy + gy

        // Mouse click to move
        if _input_mouse_pressed(0)
            playerX = _input_mouse_x()
            playerY = _input_mouse_y()

        // Normalize diagonal movement
        number len = math.sqrt(dx * dx + dy * dy)
        if len > 1.0
            dx = dx / len
            dy = dy / len

        // Apply movement
        playerX = playerX + dx * speed * dt
        playerY = playerY + dy * speed * dt

        // Render
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_circle_filled(canvasId, playerX, playerY, 20.0, "#3498db")
```

### Input Bridge Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_input_mouse_x` | none | `number` | Mouse X position |
| `_input_mouse_y` | none | `number` | Mouse Y position |
| `_input_mouse_down` | `button: integer` | `boolean` | Button held |
| `_input_mouse_pressed` | `button: integer` | `boolean` | Button just pressed |
| `_input_mouse_released` | `button: integer` | `boolean` | Button just released |
| `_input_scroll_x` | none | `number` | Horizontal scroll delta |
| `_input_scroll_y` | none | `number` | Vertical scroll delta |
| `_input_key_down` | `key: string` | `boolean` | Key held |
| `_input_key_pressed` | `key: string` | `boolean` | Key just pressed |
| `_input_key_released` | `key: string` | `boolean` | Key just released |
| `_input_get_typed` | none | `string` | Last typed character |
| `_input_touch_count` | none | `integer` | Active touch count |
| `_input_touch_x` | `index: integer` | `number` | Touch X position |
| `_input_touch_y` | `index: integer` | `number` | Touch Y position |
| `_input_touch_id` | `index: integer` | `integer` | Touch identifier |
| `_input_touch_started` | `index: integer` | `boolean` | Touch just started |
| `_input_touch_ended` | `index: integer` | `boolean` | Touch just ended |
| `_input_gamepad_connected` | `index: integer` | `boolean` | Gamepad present |
| `_input_gamepad_button` | `index, button` | `boolean` | Button pressed |
| `_input_gamepad_axis` | `index, axis` | `number` | Axis value (-1 to 1) |
| `_input_gamepad_vibrate` | `index, duration, weak, strong` | `integer` | Trigger vibration |

---

## 18. Collision Detection

Built-in collision detection helpers for common game scenarios.

### Point Collision

```clean
// Point vs Circle
boolean hit = _collision_point_circle(px, py, cx, cy, radius)

// Point vs Rectangle
boolean hit = _collision_point_rect(px, py, rx, ry, rw, rh)

// Point vs Line (with thickness)
boolean hit = _collision_point_line(px, py, x1, y1, x2, y2, thickness)
```

### Shape Collision

```clean
// Circle vs Circle
boolean hit = _collision_circle_circle(x1, y1, r1, x2, y2, r2)

// Rectangle vs Rectangle (AABB)
boolean hit = _collision_rect_rect(x1, y1, w1, h1, x2, y2, w2, h2)

// Circle vs Rectangle
boolean hit = _collision_circle_rect(cx, cy, radius, rx, ry, rw, rh)

// Line vs Line
boolean hit = _collision_line_line(ax1, ay1, ax2, ay2, bx1, by1, bx2, by2)

// Line vs Circle
boolean hit = _collision_line_circle(x1, y1, x2, y2, cx, cy, radius)

// Line vs Rectangle
boolean hit = _collision_line_rect(x1, y1, x2, y2, rx, ry, rw, rh)
```

### Overlap Resolution

```clean
// Get overlap amount between circles (for pushing apart)
number overlap = _collision_circle_circle_overlap(x1, y1, r1, x2, y2, r2)

// Get overlap amount between rectangles
number overlapX = _collision_rect_rect_overlap_x(x1, y1, w1, h1, x2, y2, w2, h2)
number overlapY = _collision_rect_rect_overlap_y(x1, y1, w1, h1, x2, y2, w2, h2)
```

### Raycast

```clean
// Cast ray and get hit distance (returns -1 if no hit)
number distance = _collision_raycast_circle(rayX, rayY, rayDirX, rayDirY, cx, cy, radius)
number distance = _collision_raycast_rect(rayX, rayY, rayDirX, rayDirY, rx, ry, rw, rh)
```

### Example: Platformer Collision

```clean
state:
    number playerX = 100.0
    number playerY = 100.0
    number playerW = 32.0
    number playerH = 48.0
    number velocityY = 0.0
    boolean onGround = false

    // Platforms (x, y, width, height)
    list<number> platformsX = [0.0, 200.0, 400.0]
    list<number> platformsY = [500.0, 400.0, 300.0]
    list<number> platformsW = [800.0, 150.0, 200.0]
    list<number> platformsH = [100.0, 20.0, 20.0]

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        // Apply gravity
        velocityY = velocityY + 800.0 * dt
        playerY = playerY + velocityY * dt
        onGround = false

        // Check collision with platforms
        integer i = 0
        while i < platformsX.length()
            if _collision_rect_rect(playerX, playerY, playerW, playerH,
                                    platformsX.get(i), platformsY.get(i),
                                    platformsW.get(i), platformsH.get(i))
                // Resolve collision (push player up)
                number overlap = _collision_rect_rect_overlap_y(
                    playerX, playerY, playerW, playerH,
                    platformsX.get(i), platformsY.get(i),
                    platformsW.get(i), platformsH.get(i))

                if velocityY > 0.0  // Falling
                    playerY = playerY - overlap
                    velocityY = 0.0
                    onGround = true
            i = i + 1

        // Jump
        if onGround and _input_key_pressed("Space")
            velocityY = -400.0

        // Draw
        _canvas_clear_color(canvasId, "#1a1a2e")

        // Draw platforms
        i = 0
        while i < platformsX.length()
            _canvas_rect_filled(canvasId, platformsX.get(i), platformsY.get(i),
                               platformsW.get(i), platformsH.get(i), "#4a5568")
            i = i + 1

        // Draw player
        _canvas_rect_filled(canvasId, playerX, playerY, playerW, playerH, "#48bb78")
```

### Collision Bridge Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_collision_point_circle` | `px, py, cx, cy, r` | `boolean` | Point in circle |
| `_collision_point_rect` | `px, py, rx, ry, rw, rh` | `boolean` | Point in rect |
| `_collision_circle_circle` | `x1, y1, r1, x2, y2, r2` | `boolean` | Circle vs circle |
| `_collision_rect_rect` | `x1, y1, w1, h1, x2, y2, w2, h2` | `boolean` | Rect vs rect |
| `_collision_circle_rect` | `cx, cy, r, rx, ry, rw, rh` | `boolean` | Circle vs rect |
| `_collision_line_line` | `ax1, ay1, ax2, ay2, bx1, by1, bx2, by2` | `boolean` | Line vs line |
| `_collision_line_circle` | `x1, y1, x2, y2, cx, cy, r` | `boolean` | Line vs circle |
| `_collision_line_rect` | `x1, y1, x2, y2, rx, ry, rw, rh` | `boolean` | Line vs rect |
| `_collision_circle_circle_overlap` | `x1, y1, r1, x2, y2, r2` | `number` | Overlap amount |
| `_collision_rect_rect_overlap_x` | `...` | `number` | X overlap |
| `_collision_rect_rect_overlap_y` | `...` | `number` | Y overlap |
| `_collision_raycast_circle` | `rx, ry, dx, dy, cx, cy, r` | `number` | Ray hit distance |
| `_collision_raycast_rect` | `rx, ry, dx, dy, x, y, w, h` | `number` | Ray hit distance |

---

## 19. Asset Management

Manage loading and caching of game assets.

### Asset Loading

```clean
// Load individual assets
integer imgId = _asset_load_image("sprites/player.png")
integer sndId = _asset_load_sound("sounds/jump.wav")
integer musId = _asset_load_music("music/level1.mp3")

// Check if asset is loaded
boolean ready = _asset_is_loaded(assetId)
```

### Batch Loading

```clean
// Queue multiple assets for loading
_asset_queue("sprites/player.png")
_asset_queue("sprites/enemy.png")
_asset_queue("sounds/jump.wav")
_asset_queue("music/background.mp3")

// Start loading all queued assets
_asset_load_all()

// Check progress (0.0 to 1.0)
number progress = _asset_get_progress()

// Check if all assets are loaded
boolean allReady = _asset_all_loaded()
```

### Asset Retrieval

```clean
// Get loaded asset by path
integer spriteId = _asset_get("sprites/player.png")
```

### Example: Loading Screen

```clean
import:
    frame.canvas

state:
    boolean assetsLoaded = false
    number loadProgress = 0.0
    integer playerSprite = 0
    integer enemySprite = 0
    integer jumpSound = 0
    integer bgMusic = 0

start()
    // Queue all game assets
    _asset_queue("sprites/player.png")
    _asset_queue("sprites/enemy.png")
    _asset_queue("sprites/tileset.png")
    _asset_queue("sounds/jump.wav")
    _asset_queue("sounds/coin.wav")
    _asset_queue("sounds/hurt.wav")
    _asset_queue("music/level1.mp3")

    // Start loading
    _asset_load_all()

    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)

functions:
    void _frame_callback(integer canvasId)
        if not assetsLoaded
            renderLoadingScreen(canvasId)

            loadProgress = _asset_get_progress()
            if _asset_all_loaded()
                assetsLoaded = true
                onAssetsReady()
        else
            renderGame(canvasId)

    void renderLoadingScreen(integer canvasId)
        _canvas_clear_color(canvasId, "#1a1a2e")

        // Draw progress bar background
        _canvas_rect_filled(canvasId, 200.0, 280.0, 400.0, 40.0, "#2d3748")

        // Draw progress bar fill
        number fillWidth = 400.0 * loadProgress
        _canvas_rect_filled(canvasId, 200.0, 280.0, fillWidth, 40.0, "#48bb78")

        // Draw loading text
        integer percent = (loadProgress * 100.0).toInteger()
        _canvas_text(canvasId, "Loading... " + percent.toString() + "%", 340.0, 350.0, 20.0, "#ffffff")

    void onAssetsReady()
        // Get asset references
        playerSprite = _asset_get("sprites/player.png")
        enemySprite = _asset_get("sprites/enemy.png")
        jumpSound = _asset_get("sounds/jump.wav")
        bgMusic = _asset_get("music/level1.mp3")

        // Start music
        _audio_play_music(bgMusic, 0.5, true)

    void renderGame(integer canvasId)
        _canvas_clear_color(canvasId, "#1a1a2e")
        // Game rendering...
```

### Asset Bridge Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_asset_load_image` | `path: string` | `integer` | Load image immediately |
| `_asset_load_sound` | `path: string` | `integer` | Load sound immediately |
| `_asset_load_music` | `path: string` | `integer` | Load music immediately |
| `_asset_queue` | `path: string` | `integer` | Add to load queue |
| `_asset_load_all` | none | `integer` | Start loading queue |
| `_asset_get_progress` | none | `number` | Get progress (0-1) |
| `_asset_all_loaded` | none | `boolean` | Check if complete |
| `_asset_is_loaded` | `assetId: integer` | `boolean` | Check single asset |
| `_asset_get` | `path: string` | `integer` | Get loaded asset ID |
| `_asset_unload` | `assetId: integer` | `integer` | Unload asset |
| `_asset_unload_all` | none | `integer` | Unload all assets |

---

## 20. Scene Management

Manage multiple game scenes (menu, gameplay, game over, etc.).

### Scene Lifecycle

```clean
// Register a scene with callbacks
_scene_register("menu", onMenuEnter, onMenuUpdate, onMenuExit)
_scene_register("game", onGameEnter, onGameUpdate, onGameExit)
_scene_register("gameover", onGameOverEnter, onGameOverUpdate, onGameOverExit)

// Switch to a scene
_scene_switch("menu")

// Get current scene name
string current = _scene_current()
```

### Scene Implementation Pattern

Since Clean Language doesn't support function references directly, use a convention-based approach:

```clean
import:
    frame.canvas

state:
    string currentScene = "menu"

    // Menu state
    integer menuSelection = 0

    // Game state
    number playerX = 400.0
    number playerY = 300.0
    integer score = 0

    // Game over state
    integer finalScore = 0

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        if currentScene == "menu"
            updateMenu(canvasId, dt)
        if currentScene == "game"
            updateGame(canvasId, dt)
        if currentScene == "gameover"
            updateGameOver(canvasId, dt)

    // ========== MENU SCENE ==========
    void switchToMenu()
        currentScene = "menu"
        menuSelection = 0

    void updateMenu(integer canvasId, number dt)
        // Input
        if _input_key_pressed("ArrowUp")
            menuSelection = menuSelection - 1
            if menuSelection < 0
                menuSelection = 2
        if _input_key_pressed("ArrowDown")
            menuSelection = (menuSelection + 1) % 3
        if _input_key_pressed("Enter")
            if menuSelection == 0
                switchToGame()
            if menuSelection == 1
                // Options (not implemented)
            if menuSelection == 2
                // Quit (not implemented)

        // Render
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_text(canvasId, "MY GAME", 320.0, 150.0, 48.0, "#ffffff")

        string color0 = "#888888"
        string color1 = "#888888"
        string color2 = "#888888"
        if menuSelection == 0
            color0 = "#48bb78"
        if menuSelection == 1
            color1 = "#48bb78"
        if menuSelection == 2
            color2 = "#48bb78"

        _canvas_text(canvasId, "Start Game", 340.0, 300.0, 24.0, color0)
        _canvas_text(canvasId, "Options", 360.0, 340.0, 24.0, color1)
        _canvas_text(canvasId, "Quit", 375.0, 380.0, 24.0, color2)

    // ========== GAME SCENE ==========
    void switchToGame()
        currentScene = "game"
        playerX = 400.0
        playerY = 300.0
        score = 0

    void updateGame(integer canvasId, number dt)
        // Input
        if _input_key_down("ArrowLeft")
            playerX = playerX - 200.0 * dt
        if _input_key_down("ArrowRight")
            playerX = playerX + 200.0 * dt
        if _input_key_pressed("Escape")
            switchToMenu()

        // Game over condition
        score = score + 1
        if score > 1000
            finalScore = score
            switchToGameOver()

        // Render
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_circle_filled(canvasId, playerX, playerY, 20.0, "#3498db")
        _canvas_text(canvasId, "Score: " + score.toString(), 20.0, 30.0, 24.0, "#ffffff")

    // ========== GAME OVER SCENE ==========
    void switchToGameOver()
        currentScene = "gameover"

    void updateGameOver(integer canvasId, number dt)
        // Input
        if _input_key_pressed("Enter")
            switchToMenu()

        // Render
        _canvas_clear_color(canvasId, "#1a1a2e")
        _canvas_text(canvasId, "GAME OVER", 290.0, 250.0, 48.0, "#e53e3e")
        _canvas_text(canvasId, "Final Score: " + finalScore.toString(), 310.0, 320.0, 24.0, "#ffffff")
        _canvas_text(canvasId, "Press ENTER to continue", 280.0, 400.0, 18.0, "#888888")

start()
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)
```

### Scene Transitions

```clean
state:
    string currentScene = "menu"
    string nextScene = ""
    number transitionAlpha = 0.0
    boolean transitioning = false
    number transitionSpeed = 2.0

functions:
    void transitionTo(string sceneName)
        nextScene = sceneName
        transitioning = true
        transitionAlpha = 0.0

    void updateTransition(number dt)
        if transitioning
            transitionAlpha = transitionAlpha + transitionSpeed * dt

            if transitionAlpha >= 1.0 and currentScene != nextScene
                currentScene = nextScene

            if transitionAlpha >= 2.0
                transitioning = false
                transitionAlpha = 0.0

    void drawTransition(integer canvasId)
        if transitioning
            number alpha = 0.0
            if transitionAlpha < 1.0
                alpha = transitionAlpha
            else
                alpha = 2.0 - transitionAlpha

            // Draw fade overlay
            integer alphaInt = (alpha * 255.0).toInteger()
            string alphaHex = alphaInt.toHex()
            if alphaHex.length() == 1
                alphaHex = "0" + alphaHex
            _canvas_rect_filled(canvasId, 0.0, 0.0, 800.0, 600.0, "#000000" + alphaHex)
```

---

## 21. Camera & Viewport

Manage scrolling game worlds and camera effects.

### Camera State

```clean
state:
    number cameraX = 0.0
    number cameraY = 0.0
    number cameraZoom = 1.0
    number cameraRotation = 0.0
```

### Camera Bridge Functions

```clean
// Set camera position
_camera_set_position(cameraX, cameraY)

// Set camera zoom (1.0 = normal)
_camera_set_zoom(zoom)

// Set camera rotation (degrees)
_camera_set_rotation(degrees)

// Apply camera transform (call before drawing world)
_camera_apply(canvasId)

// Reset camera (call before drawing UI)
_camera_reset(canvasId)

// Convert screen coordinates to world coordinates
number worldX = _camera_screen_to_world_x(screenX)
number worldY = _camera_screen_to_world_y(screenY)

// Convert world coordinates to screen coordinates
number screenX = _camera_world_to_screen_x(worldX)
number screenY = _camera_world_to_screen_y(worldY)
```

### Camera Following

```clean
state:
    number cameraX = 0.0
    number cameraY = 0.0
    number playerX = 400.0
    number playerY = 300.0
    number cameraSmooth = 5.0

functions:
    void updateCamera(number dt)
        // Target position (center player on screen)
        number targetX = playerX - 400.0
        number targetY = playerY - 300.0

        // Smooth follow
        cameraX = cameraX + (targetX - cameraX) * cameraSmooth * dt
        cameraY = cameraY + (targetY - cameraY) * cameraSmooth * dt

        // Optional: Clamp to world bounds
        if cameraX < 0.0
            cameraX = 0.0
        if cameraY < 0.0
            cameraY = 0.0
        if cameraX > worldWidth - 800.0
            cameraX = worldWidth - 800.0
        if cameraY > worldHeight - 600.0
            cameraY = worldHeight - 600.0
```

### Camera Shake

```clean
state:
    number shakeIntensity = 0.0
    number shakeDuration = 0.0
    number shakeOffsetX = 0.0
    number shakeOffsetY = 0.0

functions:
    void startShake(number intensity, number duration)
        shakeIntensity = intensity
        shakeDuration = duration

    void updateShake(number dt)
        if shakeDuration > 0.0
            shakeDuration = shakeDuration - dt
            shakeOffsetX = (math.random() - 0.5) * 2.0 * shakeIntensity
            shakeOffsetY = (math.random() - 0.5) * 2.0 * shakeIntensity

            // Decay intensity
            shakeIntensity = shakeIntensity * 0.95
        else
            shakeOffsetX = 0.0
            shakeOffsetY = 0.0
            shakeIntensity = 0.0
```

### Example: Scrolling World

```clean
import:
    frame.canvas

state:
    // World size
    number worldWidth = 2000.0
    number worldHeight = 1500.0

    // Player
    number playerX = 400.0
    number playerY = 300.0
    number speed = 300.0

    // Camera
    number cameraX = 0.0
    number cameraY = 0.0

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        // Player movement
        if _input_key_down("ArrowLeft")
            playerX = playerX - speed * dt
        if _input_key_down("ArrowRight")
            playerX = playerX + speed * dt
        if _input_key_down("ArrowUp")
            playerY = playerY - speed * dt
        if _input_key_down("ArrowDown")
            playerY = playerY + speed * dt

        // Clamp player to world
        if playerX < 0.0
            playerX = 0.0
        if playerX > worldWidth
            playerX = worldWidth
        if playerY < 0.0
            playerY = 0.0
        if playerY > worldHeight
            playerY = worldHeight

        // Camera follows player
        cameraX = playerX - 400.0
        cameraY = playerY - 300.0

        // Clamp camera
        if cameraX < 0.0
            cameraX = 0.0
        if cameraX > worldWidth - 800.0
            cameraX = worldWidth - 800.0
        if cameraY < 0.0
            cameraY = 0.0
        if cameraY > worldHeight - 600.0
            cameraY = worldHeight - 600.0

        // Render
        _canvas_clear_color(canvasId, "#1a1a2e")

        // Apply camera transform
        _canvas_save(canvasId)
        _canvas_translate(canvasId, -cameraX, -cameraY)

        // Draw world objects
        drawWorld(canvasId)

        // Draw player
        _canvas_circle_filled(canvasId, playerX, playerY, 20.0, "#3498db")

        // Reset camera for UI
        _canvas_restore(canvasId)

        // Draw UI (fixed position)
        drawUI(canvasId)

    void drawWorld(integer canvasId)
        // Draw world boundary
        _canvas_rect(canvasId, 0.0, 0.0, worldWidth, worldHeight, "#4a5568")

        // Draw some objects in world space
        _canvas_circle_filled(canvasId, 200.0, 200.0, 30.0, "#e53e3e")
        _canvas_circle_filled(canvasId, 1000.0, 500.0, 30.0, "#48bb78")
        _canvas_circle_filled(canvasId, 1800.0, 1200.0, 30.0, "#ecc94b")

    void drawUI(integer canvasId)
        _canvas_text(canvasId, "Use arrows to move", 20.0, 30.0, 18.0, "#ffffff")
        _canvas_text(canvasId, "X: " + playerX.toInteger().toString() + " Y: " + playerY.toInteger().toString(), 20.0, 55.0, 14.0, "#888888")

start()
    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)
```

### Camera Bridge Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_camera_set_position` | `x, y: number` | `integer` | Set camera position |
| `_camera_set_zoom` | `zoom: number` | `integer` | Set zoom level |
| `_camera_set_rotation` | `degrees: number` | `integer` | Set rotation |
| `_camera_apply` | `canvasId: integer` | `integer` | Apply camera transform |
| `_camera_reset` | `canvasId: integer` | `integer` | Reset to identity |
| `_camera_screen_to_world_x` | `screenX: number` | `number` | Convert screen to world |
| `_camera_screen_to_world_y` | `screenY: number` | `number` | Convert screen to world |
| `_camera_world_to_screen_x` | `worldX: number` | `number` | Convert world to screen |
| `_camera_world_to_screen_y` | `worldY: number` | `number` | Convert world to screen |

---

## 22. Advanced Rendering

### Gradients

```clean
// Create linear gradient
integer gradientId = _gradient_create_linear(x1, y1, x2, y2)

// Create radial gradient
integer gradientId = _gradient_create_radial(cx, cy, innerRadius, outerRadius)

// Add color stops (position 0.0 to 1.0)
_gradient_add_stop(gradientId, position, color)

// Set gradient as fill style
_canvas_set_fill_gradient(canvasId, gradientId)

// Set gradient as stroke style
_canvas_set_stroke_gradient(canvasId, gradientId)
```

### Gradient Example

```clean
functions:
    void drawSunset(integer canvasId)
        // Create sky gradient
        integer skyGradient = _gradient_create_linear(0.0, 0.0, 0.0, 600.0)
        _gradient_add_stop(skyGradient, 0.0, "#1a1a2e")
        _gradient_add_stop(skyGradient, 0.4, "#4a1a5c")
        _gradient_add_stop(skyGradient, 0.7, "#c94b4b")
        _gradient_add_stop(skyGradient, 1.0, "#f5af19")

        _canvas_set_fill_gradient(canvasId, skyGradient)
        _canvas_rect_filled(canvasId, 0.0, 0.0, 800.0, 600.0, "")

        // Create sun with radial gradient
        integer sunGradient = _gradient_create_radial(400.0, 450.0, 0.0, 80.0)
        _gradient_add_stop(sunGradient, 0.0, "#fffde4")
        _gradient_add_stop(sunGradient, 0.5, "#f5af19")
        _gradient_add_stop(sunGradient, 1.0, "#f5af1900")

        _canvas_set_fill_gradient(canvasId, sunGradient)
        _canvas_circle_filled(canvasId, 400.0, 450.0, 80.0, "")
```

### Paths

```clean
// Begin a new path
_path_begin(canvasId)

// Move to point (no line)
_path_move_to(canvasId, x, y)

// Line to point
_path_line_to(canvasId, x, y)

// Quadratic bezier curve
_path_quadratic_to(canvasId, cpx, cpy, x, y)

// Cubic bezier curve
_path_bezier_to(canvasId, cp1x, cp1y, cp2x, cp2y, x, y)

// Arc (for curves)
_path_arc(canvasId, cx, cy, radius, startAngle, endAngle)

// Arc to (for rounded corners)
_path_arc_to(canvasId, x1, y1, x2, y2, radius)

// Close path (line back to start)
_path_close(canvasId)

// Fill the path
_path_fill(canvasId, color)

// Stroke the path
_path_stroke(canvasId, color, lineWidth)
```

### Path Example: Custom Shapes

```clean
functions:
    void drawStar(integer canvasId, number x, number y, number outerR, number innerR, integer points, string color)
        _path_begin(canvasId)

        integer i = 0
        while i < points * 2
            number angle = (i.toNumber() * 3.14159 / points.toNumber()) - 1.5708
            number r = innerR
            if i % 2 == 0
                r = outerR

            number px = x + math.cos(angle) * r
            number py = y + math.sin(angle) * r

            if i == 0
                _path_move_to(canvasId, px, py)
            else
                _path_line_to(canvasId, px, py)

            i = i + 1

        _path_close(canvasId)
        _path_fill(canvasId, color)

    void drawRoundedRect(integer canvasId, number x, number y, number w, number h, number r, string color)
        _path_begin(canvasId)
        _path_move_to(canvasId, x + r, y)
        _path_line_to(canvasId, x + w - r, y)
        _path_arc_to(canvasId, x + w, y, x + w, y + r, r)
        _path_line_to(canvasId, x + w, y + h - r)
        _path_arc_to(canvasId, x + w, y + h, x + w - r, y + h, r)
        _path_line_to(canvasId, x + r, y + h)
        _path_arc_to(canvasId, x, y + h, x, y + h - r, r)
        _path_line_to(canvasId, x, y + r)
        _path_arc_to(canvasId, x, y, x + r, y, r)
        _path_close(canvasId)
        _path_fill(canvasId, color)
```

### Blend Modes

```clean
// Set blend mode for subsequent drawing
_canvas_set_blend_mode(canvasId, mode)

// Available modes:
// "source-over" (default) - normal blending
// "multiply" - darken
// "screen" - lighten
// "overlay" - contrast
// "darken" - min
// "lighten" - max
// "color-dodge" - brighten
// "color-burn" - darken
// "hard-light" - high contrast
// "soft-light" - soft contrast
// "difference" - invert
// "exclusion" - soft invert
// "hue" - hue blend
// "saturation" - saturation blend
// "color" - color blend
// "luminosity" - luminosity blend
```

### Alpha/Opacity

```clean
// Set global alpha for subsequent drawing
_canvas_set_alpha(canvasId, alpha)  // 0.0 to 1.0

// Example: Draw transparent overlay
_canvas_set_alpha(canvasId, 0.5)
_canvas_rect_filled(canvasId, 0.0, 0.0, 800.0, 600.0, "#000000")
_canvas_set_alpha(canvasId, 1.0)  // Reset
```

### Shadows

```clean
// Enable shadow for subsequent drawing
_canvas_set_shadow(canvasId, offsetX, offsetY, blur, color)

// Disable shadow
_canvas_clear_shadow(canvasId)

// Example
_canvas_set_shadow(canvasId, 4.0, 4.0, 10.0, "#00000080")
_canvas_rect_filled(canvasId, 100.0, 100.0, 200.0, 150.0, "#3498db")
_canvas_clear_shadow(canvasId)
```

### Text Measurement

```clean
// Get text width for alignment
number width = _canvas_measure_text(canvasId, text, fontSize)

// Example: Center text
string text = "Centered Text"
number textWidth = _canvas_measure_text(canvasId, text, 24.0)
number x = (800.0 - textWidth) / 2.0
_canvas_text(canvasId, text, x, 300.0, 24.0, "#ffffff")
```

### Advanced Rendering Bridge Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `_gradient_create_linear` | `x1, y1, x2, y2` | `integer` | Create linear gradient |
| `_gradient_create_radial` | `cx, cy, r1, r2` | `integer` | Create radial gradient |
| `_gradient_add_stop` | `gradientId, pos, color` | `integer` | Add color stop |
| `_canvas_set_fill_gradient` | `canvasId, gradientId` | `integer` | Set fill to gradient |
| `_canvas_set_stroke_gradient` | `canvasId, gradientId` | `integer` | Set stroke to gradient |
| `_path_begin` | `canvasId` | `integer` | Begin path |
| `_path_move_to` | `canvasId, x, y` | `integer` | Move to point |
| `_path_line_to` | `canvasId, x, y` | `integer` | Line to point |
| `_path_quadratic_to` | `canvasId, cpx, cpy, x, y` | `integer` | Quadratic curve |
| `_path_bezier_to` | `canvasId, cp1x, cp1y, cp2x, cp2y, x, y` | `integer` | Bezier curve |
| `_path_arc` | `canvasId, cx, cy, r, start, end` | `integer` | Arc |
| `_path_arc_to` | `canvasId, x1, y1, x2, y2, r` | `integer` | Arc to |
| `_path_close` | `canvasId` | `integer` | Close path |
| `_path_fill` | `canvasId, color` | `integer` | Fill path |
| `_path_stroke` | `canvasId, color, width` | `integer` | Stroke path |
| `_canvas_set_blend_mode` | `canvasId, mode` | `integer` | Set blend mode |
| `_canvas_set_alpha` | `canvasId, alpha` | `integer` | Set global alpha |
| `_canvas_set_shadow` | `canvasId, ox, oy, blur, color` | `integer` | Enable shadow |
| `_canvas_clear_shadow` | `canvasId` | `integer` | Disable shadow |
| `_canvas_measure_text` | `canvasId, text, size` | `number` | Get text width |

---

## 23. Tween & Easing

Helper functions for smooth animations.

### Easing Functions

```clean
// Get eased value (t from 0.0 to 1.0)
number eased = _ease_linear(t)
number eased = _ease_in_quad(t)
number eased = _ease_out_quad(t)
number eased = _ease_in_out_quad(t)
number eased = _ease_in_cubic(t)
number eased = _ease_out_cubic(t)
number eased = _ease_in_out_cubic(t)
number eased = _ease_in_quart(t)
number eased = _ease_out_quart(t)
number eased = _ease_in_out_quart(t)
number eased = _ease_in_sine(t)
number eased = _ease_out_sine(t)
number eased = _ease_in_out_sine(t)
number eased = _ease_in_expo(t)
number eased = _ease_out_expo(t)
number eased = _ease_in_out_expo(t)
number eased = _ease_in_elastic(t)
number eased = _ease_out_elastic(t)
number eased = _ease_in_out_elastic(t)
number eased = _ease_in_bounce(t)
number eased = _ease_out_bounce(t)
number eased = _ease_in_out_bounce(t)
number eased = _ease_in_back(t)
number eased = _ease_out_back(t)
number eased = _ease_in_out_back(t)
```

### Tween Helper Class

```clean
class Tween
    number startValue
    number endValue
    number duration
    number elapsed
    string easingType
    boolean finished
    number currentValue

    constructor(number from, number to, number dur, string easing)
        startValue = from
        endValue = to
        duration = dur
        elapsed = 0.0
        easingType = easing
        finished = false
        currentValue = from

    functions:
        void update(number dt)
            if finished
                return

            elapsed = elapsed + dt
            if elapsed >= duration
                elapsed = duration
                finished = true

            number t = elapsed / duration
            number eased = applyEasing(t)
            currentValue = startValue + (endValue - startValue) * eased

        number applyEasing(number t)
            if easingType == "linear"
                return t
            if easingType == "easeInQuad"
                return t * t
            if easingType == "easeOutQuad"
                return t * (2.0 - t)
            if easingType == "easeInOutQuad"
                if t < 0.5
                    return 2.0 * t * t
                return -1.0 + (4.0 - 2.0 * t) * t
            if easingType == "easeOutBounce"
                return _ease_out_bounce(t)
            // Default to linear
            return t

        void reset()
            elapsed = 0.0
            finished = false
            currentValue = startValue
```

### Example: Animated UI

```clean
state:
    Tween menuTween = null
    number menuY = -200.0

start()
    // Animate menu sliding in
    menuTween = Tween(-200.0, 100.0, 0.5, "easeOutBounce")

    integer canvasId = _canvas_init(800, 600)
    _canvas_request_frame(canvasId)

functions:
    void _frame_callback(integer canvasId)
        number dt = _canvas_get_delta_time()

        // Update tween
        menuTween.update(dt)
        menuY = menuTween.currentValue

        // Draw
        _canvas_clear_color(canvasId, "#1a1a2e")

        // Draw animated menu
        _canvas_rect_filled(canvasId, 250.0, menuY, 300.0, 400.0, "#2d3748")
        _canvas_text(canvasId, "MENU", 360.0, menuY + 50.0, 32.0, "#ffffff")
```

---

## 24. Complete Plugin Manifest

Updated `plugin.toml` with all bridge functions:

```toml
[plugin]
name = "frame.canvas"
version = "2.0.0"
description = "Complete canvas rendering, animation, and game development plugin for Clean Language"
author = "Clean Language Team"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"
get_keywords = "get_keywords"

[handles]
blocks = ["canvasScene", "draw", "onFrame"]

[paths]
owns = ["src/canvas"]
auto_create = true
patterns = ["*.cln"]
implicit_import = true

[bridge]
functions = [
  # ===== Canvas Lifecycle =====
  { name = "_canvas_init", params = ["integer", "integer"], returns = "integer" },
  { name = "_canvas_clear", params = ["integer"], returns = "integer" },
  { name = "_canvas_clear_color", params = ["integer", "string"], returns = "integer" },
  { name = "_canvas_present", params = ["integer"], returns = "integer" },

  # ===== Basic Shapes =====
  { name = "_canvas_circle", params = ["integer", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_circle_filled", params = ["integer", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_rect", params = ["integer", "number", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_rect_filled", params = ["integer", "number", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_line", params = ["integer", "number", "number", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_text", params = ["integer", "string", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_image", params = ["integer", "string", "number", "number", "number", "number"], returns = "integer" },

  # ===== Transforms =====
  { name = "_canvas_save", params = ["integer"], returns = "integer" },
  { name = "_canvas_restore", params = ["integer"], returns = "integer" },
  { name = "_canvas_translate", params = ["integer", "number", "number"], returns = "integer" },
  { name = "_canvas_rotate", params = ["integer", "number"], returns = "integer" },
  { name = "_canvas_scale", params = ["integer", "number", "number"], returns = "integer" },

  # ===== Animation =====
  { name = "_canvas_request_frame", params = ["integer"], returns = "integer" },
  { name = "_canvas_cancel_frame", params = ["integer"], returns = "integer" },
  { name = "_canvas_get_delta_time", params = [], returns = "number" },

  # ===== Audio =====
  { name = "_audio_load_sound", params = ["string"], returns = "integer" },
  { name = "_audio_load_music", params = ["string"], returns = "integer" },
  { name = "_audio_play", params = ["integer", "number", "boolean"], returns = "integer" },
  { name = "_audio_play_at", params = ["integer", "number", "number", "number"], returns = "integer" },
  { name = "_audio_stop", params = ["integer"], returns = "integer" },
  { name = "_audio_stop_all", params = [], returns = "integer" },
  { name = "_audio_play_music", params = ["integer", "number", "boolean"], returns = "integer" },
  { name = "_audio_pause_music", params = [], returns = "integer" },
  { name = "_audio_resume_music", params = [], returns = "integer" },
  { name = "_audio_stop_music", params = [], returns = "integer" },
  { name = "_audio_set_music_volume", params = ["number"], returns = "integer" },
  { name = "_audio_set_master_volume", params = ["number"], returns = "integer" },
  { name = "_audio_is_playing", params = ["integer"], returns = "boolean" },
  { name = "_audio_set_listener", params = ["number", "number"], returns = "integer" },

  # ===== Sprites =====
  { name = "_sprite_load", params = ["string"], returns = "integer" },
  { name = "_sprite_load_sheet", params = ["string", "integer", "integer"], returns = "integer" },
  { name = "_sprite_draw", params = ["integer", "integer", "number", "number"], returns = "integer" },
  { name = "_sprite_draw_sized", params = ["integer", "integer", "number", "number", "number", "number"], returns = "integer" },
  { name = "_sprite_draw_ex", params = ["integer", "integer", "number", "number", "number", "number", "number", "number", "number"], returns = "integer" },
  { name = "_sprite_draw_frame", params = ["integer", "integer", "integer", "number", "number"], returns = "integer" },
  { name = "_sprite_draw_frame_sized", params = ["integer", "integer", "integer", "number", "number", "number", "number"], returns = "integer" },
  { name = "_sprite_get_frame_count", params = ["integer"], returns = "integer" },
  { name = "_sprite_get_width", params = ["integer"], returns = "integer" },
  { name = "_sprite_get_height", params = ["integer"], returns = "integer" },

  # ===== Input: Mouse =====
  { name = "_input_mouse_x", params = [], returns = "number" },
  { name = "_input_mouse_y", params = [], returns = "number" },
  { name = "_input_mouse_down", params = ["integer"], returns = "boolean" },
  { name = "_input_mouse_pressed", params = ["integer"], returns = "boolean" },
  { name = "_input_mouse_released", params = ["integer"], returns = "boolean" },
  { name = "_input_scroll_x", params = [], returns = "number" },
  { name = "_input_scroll_y", params = [], returns = "number" },

  # ===== Input: Keyboard =====
  { name = "_input_key_down", params = ["string"], returns = "boolean" },
  { name = "_input_key_pressed", params = ["string"], returns = "boolean" },
  { name = "_input_key_released", params = ["string"], returns = "boolean" },
  { name = "_input_get_typed", params = [], returns = "string" },

  # ===== Input: Touch =====
  { name = "_input_touch_count", params = [], returns = "integer" },
  { name = "_input_touch_x", params = ["integer"], returns = "number" },
  { name = "_input_touch_y", params = ["integer"], returns = "number" },
  { name = "_input_touch_id", params = ["integer"], returns = "integer" },
  { name = "_input_touch_started", params = ["integer"], returns = "boolean" },
  { name = "_input_touch_ended", params = ["integer"], returns = "boolean" },

  # ===== Input: Gamepad =====
  { name = "_input_gamepad_connected", params = ["integer"], returns = "boolean" },
  { name = "_input_gamepad_button", params = ["integer", "integer"], returns = "boolean" },
  { name = "_input_gamepad_axis", params = ["integer", "integer"], returns = "number" },
  { name = "_input_gamepad_vibrate", params = ["integer", "number", "number", "number"], returns = "integer" },

  # ===== Collision =====
  { name = "_collision_point_circle", params = ["number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_point_rect", params = ["number", "number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_circle_circle", params = ["number", "number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_rect_rect", params = ["number", "number", "number", "number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_circle_rect", params = ["number", "number", "number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_line_line", params = ["number", "number", "number", "number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_line_circle", params = ["number", "number", "number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_line_rect", params = ["number", "number", "number", "number", "number", "number", "number", "number"], returns = "boolean" },
  { name = "_collision_circle_circle_overlap", params = ["number", "number", "number", "number", "number", "number"], returns = "number" },
  { name = "_collision_rect_rect_overlap_x", params = ["number", "number", "number", "number", "number", "number", "number", "number"], returns = "number" },
  { name = "_collision_rect_rect_overlap_y", params = ["number", "number", "number", "number", "number", "number", "number", "number"], returns = "number" },
  { name = "_collision_raycast_circle", params = ["number", "number", "number", "number", "number", "number", "number"], returns = "number" },
  { name = "_collision_raycast_rect", params = ["number", "number", "number", "number", "number", "number", "number", "number"], returns = "number" },

  # ===== Assets =====
  { name = "_asset_load_image", params = ["string"], returns = "integer" },
  { name = "_asset_load_sound", params = ["string"], returns = "integer" },
  { name = "_asset_load_music", params = ["string"], returns = "integer" },
  { name = "_asset_queue", params = ["string"], returns = "integer" },
  { name = "_asset_load_all", params = [], returns = "integer" },
  { name = "_asset_get_progress", params = [], returns = "number" },
  { name = "_asset_all_loaded", params = [], returns = "boolean" },
  { name = "_asset_is_loaded", params = ["integer"], returns = "boolean" },
  { name = "_asset_get", params = ["string"], returns = "integer" },
  { name = "_asset_unload", params = ["integer"], returns = "integer" },
  { name = "_asset_unload_all", params = [], returns = "integer" },

  # ===== Camera =====
  { name = "_camera_set_position", params = ["number", "number"], returns = "integer" },
  { name = "_camera_set_zoom", params = ["number"], returns = "integer" },
  { name = "_camera_set_rotation", params = ["number"], returns = "integer" },
  { name = "_camera_apply", params = ["integer"], returns = "integer" },
  { name = "_camera_reset", params = ["integer"], returns = "integer" },
  { name = "_camera_screen_to_world_x", params = ["number"], returns = "number" },
  { name = "_camera_screen_to_world_y", params = ["number"], returns = "number" },
  { name = "_camera_world_to_screen_x", params = ["number"], returns = "number" },
  { name = "_camera_world_to_screen_y", params = ["number"], returns = "number" },

  # ===== Gradients =====
  { name = "_gradient_create_linear", params = ["number", "number", "number", "number"], returns = "integer" },
  { name = "_gradient_create_radial", params = ["number", "number", "number", "number"], returns = "integer" },
  { name = "_gradient_add_stop", params = ["integer", "number", "string"], returns = "integer" },
  { name = "_canvas_set_fill_gradient", params = ["integer", "integer"], returns = "integer" },
  { name = "_canvas_set_stroke_gradient", params = ["integer", "integer"], returns = "integer" },

  # ===== Paths =====
  { name = "_path_begin", params = ["integer"], returns = "integer" },
  { name = "_path_move_to", params = ["integer", "number", "number"], returns = "integer" },
  { name = "_path_line_to", params = ["integer", "number", "number"], returns = "integer" },
  { name = "_path_quadratic_to", params = ["integer", "number", "number", "number", "number"], returns = "integer" },
  { name = "_path_bezier_to", params = ["integer", "number", "number", "number", "number", "number", "number"], returns = "integer" },
  { name = "_path_arc", params = ["integer", "number", "number", "number", "number", "number"], returns = "integer" },
  { name = "_path_arc_to", params = ["integer", "number", "number", "number", "number", "number"], returns = "integer" },
  { name = "_path_close", params = ["integer"], returns = "integer" },
  { name = "_path_fill", params = ["integer", "string"], returns = "integer" },
  { name = "_path_stroke", params = ["integer", "string", "number"], returns = "integer" },

  # ===== Advanced Rendering =====
  { name = "_canvas_set_blend_mode", params = ["integer", "string"], returns = "integer" },
  { name = "_canvas_set_alpha", params = ["integer", "number"], returns = "integer" },
  { name = "_canvas_set_shadow", params = ["integer", "number", "number", "number", "string"], returns = "integer" },
  { name = "_canvas_clear_shadow", params = ["integer"], returns = "integer" },
  { name = "_canvas_measure_text", params = ["integer", "string", "number"], returns = "number" },

  # ===== Easing =====
  { name = "_ease_linear", params = ["number"], returns = "number" },
  { name = "_ease_in_quad", params = ["number"], returns = "number" },
  { name = "_ease_out_quad", params = ["number"], returns = "number" },
  { name = "_ease_in_out_quad", params = ["number"], returns = "number" },
  { name = "_ease_in_cubic", params = ["number"], returns = "number" },
  { name = "_ease_out_cubic", params = ["number"], returns = "number" },
  { name = "_ease_in_out_cubic", params = ["number"], returns = "number" },
  { name = "_ease_in_sine", params = ["number"], returns = "number" },
  { name = "_ease_out_sine", params = ["number"], returns = "number" },
  { name = "_ease_in_out_sine", params = ["number"], returns = "number" },
  { name = "_ease_in_expo", params = ["number"], returns = "number" },
  { name = "_ease_out_expo", params = ["number"], returns = "number" },
  { name = "_ease_in_out_expo", params = ["number"], returns = "number" },
  { name = "_ease_in_elastic", params = ["number"], returns = "number" },
  { name = "_ease_out_elastic", params = ["number"], returns = "number" },
  { name = "_ease_in_out_elastic", params = ["number"], returns = "number" },
  { name = "_ease_in_bounce", params = ["number"], returns = "number" },
  { name = "_ease_out_bounce", params = ["number"], returns = "number" },
  { name = "_ease_in_out_bounce", params = ["number"], returns = "number" },
  { name = "_ease_in_back", params = ["number"], returns = "number" },
  { name = "_ease_out_back", params = ["number"], returns = "number" },
  { name = "_ease_in_out_back", params = ["number"], returns = "number" },
]
```

---

## 25. Summary

### Feature Completeness

| Category | Features | Status |
|----------|----------|--------|
| **Core Rendering** | Shapes, text, images, transforms | Complete |
| **Animation** | Frame loop, delta time, tweening | Complete |
| **Audio** | Sound effects, music, spatial audio | Complete |
| **Sprites** | Sprite sheets, frame animation | Complete |
| **Input** | Mouse, keyboard, touch, gamepad | Complete |
| **Collision** | Point, shape, raycast | Complete |
| **Assets** | Loading, caching, progress | Complete |
| **Scenes** | State-based scene management | Complete |
| **Camera** | Scrolling, zoom, shake | Complete |
| **Advanced** | Gradients, paths, blend modes | Complete |

### What You Can Build

With this specification, you can build:

- **Platformers** (Mario-style) - sprites, collision, camera scrolling
- **Shooters** (top-down, side-scrolling) - input, collision, particles
- **Puzzle Games** - touch input, tweening, state management
- **Racing Games** - camera follow, collision, audio
- **RPGs** - scene management, sprites, dialogue (DOM overlay)
- **Arcade Games** - all features combined
- **Data Visualizations** - gradients, paths, animations
- **Interactive Art** - paths, blend modes, input

### No Compiler Changes Required

All 100+ bridge functions are implemented in:
- **plugin.toml** - Function declarations
- **canvas-bridge.js** - JavaScript runtime

### File Locations

| Item | Location |
|------|----------|
| Plugin manifest | `~/.cleen/plugins/frame.canvas/2.0.0/plugin.toml` |
| Plugin source | `plugins/frame.canvas/src/main.cln` |
| Runtime bridge | `plugins/frame.canvas/runtime/canvas-bridge.js` |
| Examples | `plugins/frame.canvas/runtime/examples/` |

---

**End of Document 10 (v2.0 - Complete Game Development Specification)**
