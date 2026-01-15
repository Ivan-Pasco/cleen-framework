# Clean Canvas Browser Runtime

Browser runtime for Clean Language canvas applications.

## Files

| File | Description |
|------|-------------|
| `canvas-bridge.js` | WASM bridge implementing `_canvas_*` functions |
| `loader.js` | WASM module loader with canvas support |
| `example.html` | Interactive demo of canvas bridge |

## Quick Start

### 1. Include Scripts

```html
<script src="canvas-bridge.js"></script>
<script src="loader.js"></script>
```

### 2. Load WASM Module

```html
<script>
  CleanCanvas.load('app.wasm').then(app => {
    app.start();
  });
</script>
```

### 3. Create Container

```html
<div id="canvas-container"></div>
```

The canvas will be automatically created and appended to this container.

## Bridge Functions

All bridge functions are available to WASM modules:

### Lifecycle
- `_canvas_init(width, height)` - Create canvas, returns ID
- `_canvas_clear(id)` - Clear canvas
- `_canvas_clear_color(id, color)` - Clear with color
- `_canvas_present(id)` - Flush buffer (no-op for Canvas2D)

### Shapes
- `_canvas_circle(id, x, y, radius, color)` - Draw circle outline
- `_canvas_circle_filled(id, x, y, radius, color)` - Draw filled circle
- `_canvas_rect(id, x, y, w, h, color)` - Draw rectangle outline
- `_canvas_rect_filled(id, x, y, w, h, color)` - Draw filled rectangle
- `_canvas_line(id, x1, y1, x2, y2, stroke, color)` - Draw line

### Text & Images
- `_canvas_text(id, text, x, y, size, color)` - Draw text
- `_canvas_image(id, src, x, y, w, h)` - Draw image

### Transforms
- `_canvas_save(id)` - Save state
- `_canvas_restore(id)` - Restore state
- `_canvas_translate(id, x, y)` - Translate
- `_canvas_rotate(id, angle)` - Rotate (degrees)
- `_canvas_scale(id, sx, sy)` - Scale

### Animation
- `_canvas_request_frame(id)` - Request animation frame
- `_canvas_cancel_frame(frameId)` - Cancel animation
- `_canvas_get_delta_time()` - Get time since last frame (seconds)

### Input
- `_canvas_on_pointer(id, handler)` - Register pointer handler
- `_canvas_on_key(id, handler)` - Register key handler
- `_canvas_pointer_x()` - Get pointer X
- `_canvas_pointer_y()` - Get pointer Y

## Example Clean Code

```clean
import:
    frame.canvas

start()
    integer canvas = _canvas_init(800, 600)

    // Animation loop
    animate(canvas, 0.0)

animate(integer canvas, number angle)
    _canvas_clear_color(canvas, "#1a1a2e")

    // Draw rotating square
    _canvas_save(canvas)
    _canvas_translate(canvas, 400.0, 300.0)
    _canvas_rotate(canvas, angle)
    _canvas_rect_filled(canvas, -50.0, -50.0, 100.0, 100.0, "#4ecca3")
    _canvas_restore(canvas)

    // Request next frame
    number dt = _canvas_get_delta_time()
    _canvas_request_frame(canvas)
    animate(canvas, angle + 90.0 * dt)
```

## Testing

Open `example.html` in a browser to see interactive demos of:
- Basic shapes drawing
- Animation with delta time
- Transform operations (translate, rotate, scale)

## Integration with frame.ui

Canvas can be embedded in UI layouts:

```clean
import:
    frame.ui
    frame.canvas

screen "Dashboard":
    ui.column:
        ui.region target="canvas" height=400:
            canvasScene: width=800 height=400
                draw:
                    canvas.clear color="#1a1a2e"
                    canvas.circle x=400 y=200 radius=50 color="#4ecca3"

        ui.button "Click me":
            onClick:
                print("Button clicked")
```

## Browser Compatibility

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

Requires WebAssembly and Canvas 2D support.
