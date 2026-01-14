# Frame Canvas Specification (12)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 1.0.0
**Location:** `/docs/specification/12_frame_canvas.md`

---

## 1. Introduction

**Frame Canvas** is a rendering and animation plugin for Clean Language applications. It provides a declarative API for drawing shapes, text, images, and creating animations.

### Design Philosophy

- **Canvas is a render target, not a UI system** - Use frame.ui for interfaces, frame.canvas for visuals
- **Declarative drawing** - Describe what to draw, not how to draw it
- **State-driven updates** - Canvas redraws when state changes
- **Hybrid rendering** - Combine DOM (frame.ui) and Canvas seamlessly

### When to Use Canvas

| Use Canvas For | Use frame.ui For |
|----------------|------------------|
| Charts and graphs | Forms and inputs |
| Games and animations | Buttons and controls |
| Custom visualizations | Text editing |
| Visual effects | Accessibility |
| Data plots | Navigation |

---

## 2. Plugin Architecture

### Folder Convention

frame.canvas owns the `src/canvas/` folder:

```
myapp/
├── src/
│   ├── canvas/           # Owned by frame.canvas
│   │   ├── GameScene.cln
│   │   ├── ChartViz.cln
│   │   └── Animation.cln
│   ├── ui/               # Owned by frame.ui
│   └── data/             # Owned by frame.data
```

Files in `src/canvas/` automatically import frame.canvas.

### Plugin Manifest

```toml
[plugin]
name = "frame.canvas"
version = "1.0.0"

[handles]
blocks = ["canvasScene", "draw", "onFrame"]

[paths]
owns = ["src/canvas"]
auto_create = true
patterns = ["*.cln"]
implicit_import = true
```

---

## 3. Canvas Scene

A `canvasScene` block creates a drawing surface.

### Basic Syntax

```clean
canvasScene: width=800 height=600
    draw:
        canvas.clear
        canvas.circle x=100 y=100 radius=50 color="blue"
```

### Attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `width` | integer | 800 | Canvas width in pixels |
| `height` | integer | 600 | Canvas height in pixels |
| `id` | string | "_canvas_0" | Canvas identifier |

---

## 4. Drawing Commands

### 4.1 Clear

Clear the canvas before drawing.

```clean
canvas.clear
canvas.clear color="white"
```

### 4.2 Circle

Draw a circle.

```clean
canvas.circle x=100 y=100 radius=50 color="blue"
canvas.circle x=100 y=100 radius=50 color="red" filled=false
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 0 | Center X position |
| `y` | number | 0 | Center Y position |
| `radius` | number | 10 | Circle radius |
| `color` | string | "black" | Fill/stroke color |
| `filled` | boolean | true | Filled or outline |

### 4.3 Rectangle

Draw a rectangle.

```clean
canvas.rect x=20 y=20 width=100 height=60 color="green"
canvas.rect x=20 y=20 width=100 height=60 color="green" filled=false
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 0 | Top-left X position |
| `y` | number | 0 | Top-left Y position |
| `width` | number | 100 | Rectangle width |
| `height` | number | 100 | Rectangle height |
| `color` | string | "black" | Fill/stroke color |
| `filled` | boolean | true | Filled or outline |

### 4.4 Line

Draw a line.

```clean
canvas.line fromX=0 fromY=0 toX=100 toY=100 stroke=2 color="black"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `fromX` | number | 0 | Start X position |
| `fromY` | number | 0 | Start Y position |
| `toX` | number | 100 | End X position |
| `toY` | number | 100 | End Y position |
| `stroke` | number | 1 | Line width |
| `color` | string | "black" | Line color |

### 4.5 Text

Draw text (visual only, not selectable).

```clean
canvas.text value="Hello World" x=50 y=100 size=24 color="black"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | string | "" | Text to display |
| `x` | number | 0 | X position |
| `y` | number | 0 | Y position (baseline) |
| `size` | number | 16 | Font size |
| `color` | string | "black" | Text color |

### 4.6 Image

Draw an image.

```clean
canvas.image src="assets/logo.png" x=20 y=20 width=100 height=50
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `src` | string | "" | Image path/URL |
| `x` | number | 0 | X position |
| `y` | number | 0 | Y position |
| `width` | number | 100 | Display width |
| `height` | number | 100 | Display height |

---

## 5. Transforms

Transforms affect subsequent drawing commands.

### 5.1 Save/Restore

Save and restore canvas state.

```clean
canvas.save
canvas.rotate angle=45
canvas.circle x=0 y=0 radius=30
canvas.restore
```

### 5.2 Translate

Move the origin.

```clean
canvas.translate x=100 y=100
```

### 5.3 Rotate

Rotate around the origin.

```clean
canvas.rotate angle=45
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `angle` | number | 0 | Rotation in degrees |

### 5.4 Scale

Scale the canvas.

```clean
canvas.scale x=2 y=2
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 1 | Horizontal scale |
| `y` | number | 1 | Vertical scale |

---

## 6. Animation

### 6.1 Frame Handler

The `onFrame` block runs on each animation frame.

```clean
canvasScene: width=400 height=300

    state:
        number x = 0

    onFrame: param="dt"
        x = x + 100 * dt

    draw:
        canvas.clear
        canvas.circle x=x y=150 radius=20 color="blue"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `param` | string | "dt" | Delta time variable name |

### 6.2 Delta Time

The `dt` parameter contains time since last frame in seconds.

```clean
onFrame: param="dt"
    // dt is typically ~0.016 for 60fps
    position = position + velocity * dt
```

---

## 7. Input Handling

### 7.1 Pointer Events

Handle mouse/touch input.

```clean
canvasScene:
    onPointerDown: params="x,y"
        print("Clicked at " + x.toString() + ", " + y.toString())

    onPointerMove: params="x,y"
        cursor_x = x
        cursor_y = y
```

### 7.2 Key Events

Handle keyboard input.

```clean
canvasScene:
    onKeyDown: params="key"
        if key == "ArrowUp"
            player_y = player_y - 10
```

---

## 8. Integration with frame.ui

Canvas is typically embedded within UI layouts.

### Hybrid Screen Example

```clean
import:
    frame.ui
    frame.canvas

screen "Dashboard":

    state:
        number progress = 0.3

    ui.column:

        // Canvas region for visualization
        ui.region target="canvas" height=200:
            canvasScene:
                draw:
                    canvas.clear color="#f0f0f0"
                    canvas.rect x=50 y=50 width=300 height=20 color="#ddd"
                    canvas.rect x=50 y=50 width=(300 * progress) height=20 color="blue"

        // UI controls below
        ui.button "Increase":
            onClick:
                progress = min(progress + 0.1, 1.0)
```

---

## 9. Bridge Functions

Canvas commands compile to bridge function calls:

| Command | Bridge Function |
|---------|-----------------|
| `canvas.clear` | `_canvas_clear(canvas_id)` |
| `canvas.circle` | `_canvas_circle_filled(canvas_id, x, y, radius, color)` |
| `canvas.rect` | `_canvas_rect_filled(canvas_id, x, y, w, h, color)` |
| `canvas.line` | `_canvas_line(canvas_id, x1, y1, x2, y2, stroke, color)` |
| `canvas.text` | `_canvas_text(canvas_id, text, x, y, size, color)` |
| `canvas.image` | `_canvas_image(canvas_id, src, x, y, w, h)` |
| `canvas.save` | `_canvas_save(canvas_id)` |
| `canvas.restore` | `_canvas_restore(canvas_id)` |
| `canvas.translate` | `_canvas_translate(canvas_id, x, y)` |
| `canvas.rotate` | `_canvas_rotate(canvas_id, angle)` |
| `canvas.scale` | `_canvas_scale(canvas_id, sx, sy)` |

---

## 10. Complete Example

```clean
// src/canvas/SpinningSquare.cln
// No import needed - in src/canvas/ folder

screen "SpinningSquare":

    state:
        number angle = 0

    canvasScene: width=400 height=400

        onFrame: param="dt"
            angle = angle + 90 * dt

        draw:
            canvas.clear color="white"

            // Move to center
            canvas.save
            canvas.translate x=200 y=200

            // Rotate
            canvas.rotate angle=angle

            // Draw centered square
            canvas.rect x=-50 y=-50 width=100 height=100 color="blue"

            canvas.restore

            // Draw frame info
            canvas.text value="Angle: " + angle.toString() x=10 y=30 size=14 color="gray"
```

---

## 11. Performance Guidelines

1. **Clear once per frame** - Don't clear multiple times
2. **Use save/restore** - Avoid transform state leaks
3. **Batch similar draws** - Group shapes by color/style
4. **Minimize state changes** - Transforms are expensive
5. **Use delta time** - Frame-rate independent animation

---

## 12. What Canvas Does NOT Do

Canvas does **not**:
- Provide text input fields
- Handle form validation
- Manage focus/accessibility
- Replace UI widgets

For these, use frame.ui.

---

## 13. Version History

| Version | Changes |
|---------|---------|
| 1.0.0 | Initial release - shapes, transforms, animation |

---

**End of Document 12**
