# Frame Canvas Specification

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 2.1.0
**Location:** `/documents/specification/12_frame_canvas.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

---

## 1. Introduction & Philosophy

**Frame Canvas** is the animation and rendering platform for Clean Language — a declarative, WebAssembly-native platform for building games, animations, data visualizations, and interactive experiences.

### Three-Layer Architecture

Frame Canvas is organized into three conceptual layers, all expressed in Clean Language:

```
Layer 3: Effects & Physics
    Particle systems, force fields, bloom, blur, glow, shadows

Layer 2: Animation System
    Tweens, timelines, animation state machines, path animation

Layer 1: Scene & Drawing
    Immediate-mode primitives, sprites, gradients, transforms, audio
```

Higher layers depend on lower layers — you can use Layer 1 without Layer 2, and Layer 2 without Layer 3. All layers compile to the same bridge function calls.

### Design Philosophy

- **Declarative first** — Describe what you want; the runtime handles when and how
- **State-driven** — Canvas redraws reflect state variables; no manual invalidation
- **Asset-driven authoring** — Assets declared once in `assets:`, reused everywhere
- **Frame-rate independent** — All animation uses delta time; targets any frame rate
- **No boilerplate** — A minimal canvas scene is four lines of code

### When to Use Canvas

| Use Frame Canvas For | Use frame.ui For |
|---------------------|-----------------|
| Games and interactive simulations | Forms, inputs, buttons |
| Custom data visualizations | Navigation, menus |
| Particle effects and animations | Text editing, accessibility |
| Physics simulations | Layout and responsive design |
| Sprite-based content | Standard web controls |
| Audio-reactive visuals | SEO-critical content |

### Plugin Ownership

Frame Canvas owns the `app/canvas/` folder. When `frame.canvas` is declared in `app.cln`, all `.cln` files inside `app/canvas/` and its subfolders are automatically processed by the plugin — no per-file import statements are needed (`implicit_import = true`).

```
myapp/
├── app/
│   ├── canvas/               // Owned by frame.canvas
│   │   ├── scenes/
│   │   │   ├── GameScene.cln
│   │   │   └── MenuScene.cln
│   │   ├── sprites/
│   │   │   └── HeroSprites.cln
│   │   └── audio/
│   │       └── SoundBank.cln
│   ├── components/           // Owned by frame.ui
│   └── data/                 // Owned by frame.data
```

---

## 2. Scene Declaration (`canvasScene:`)

Every canvas file is anchored by a `canvasScene:` block. It is the top-level container for all sub-blocks. There is exactly one `canvasScene:` per file.

### Syntax

```clean
canvasScene: width=800 height=600 fps=60 id="main"
```

### Attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `width` | integer | 800 | Canvas width in pixels |
| `height` | integer | 600 | Canvas height in pixels |
| `fps` | integer | 60 | Target frame rate |
| `id` | string | "_canvas_0" | Canvas element identifier for HTML binding |

### Sub-blocks (order is fixed)

All sub-blocks live inside the `canvasScene:` body. The canonical order is:

1. `assets:` — asset preloading declarations
2. `layers:` — named render layer declarations
3. `state:` — persistent mutable variables
4. `animSprite ...` — animated sprite clip definitions
5. `particles ...` — particle emitter definitions
6. `tween ...` — named tween definitions
7. `timeline ...` — named timeline definitions
8. `animState ...` — animation state machine definitions
9. `path ...` — named path definitions
10. `gradient ...` — named gradient definitions
11. `customEase ...` — custom cubic bezier easing declarations
12. `camera:` — camera/viewport configuration
13. `init:` — one-time initialization logic
14. `draw:` — per-frame drawing logic
15. `onFrame:` — per-frame update logic
16. Event handlers (`onPointerDown:`, `onPointerMove:`, `onPointerUp:`, `onKeyDown:`, `onKeyUp:`)
17. `onExit:` — scene teardown hook
18. `onPause:` — scene pause hook
19. `onResume:` — scene resume hook

### Minimal Example

```clean
canvasScene: width=400 height=300

	state:
		number angle = 0.0

	onFrame: param="dt"
		angle = angle + 90.0 * dt

	draw:
		canvas.clear color="#1a1a2e"
		canvas.save
		canvas.translate x=200 y=150
		canvas.rotate angle=angle
		canvas.rect x=-40 y=-40 width=80 height=80 color="#e94560"
		canvas.restore
```

### Full-Feature Example

```clean
canvasScene: width=800 height=600 fps=60 id="game"

	assets:
		spritesheet "hero" src="sprites/hero.png" frameWidth=48 frameHeight=64
		sound "jump" src="sounds/jump.wav"
		music "theme" src="music/theme.mp3"

	layers:
		layer "background" z=0
		layer "characters" z=20
		layer "hud" z=100

	state:
		number playerX = 100.0
		number playerY = 400.0
		integer score = 0

	init:
		audio.music.play "theme" loop=true volume=0.6

	draw:
		on layer "background":
			canvas.clear color="#0f0e17"
		on layer "hud":
			canvas.text value="Score: " + score.toString() x=10 y=30 size=20 color="white"

	onFrame: param="dt"
		playerX = playerX + 60.0 * dt
```

### Scene Lifecycle Hooks

Three optional hooks fire around scene transitions. They live inside the `canvasScene:` body at the same level as `init:` and `draw:`.

```clean
canvasScene: width=800 height=600

	onExit:
		// Fires when this scene is removed (scene.change or scene.pop)
		audio.music.fadeOut duration=0.5

	onPause:
		// Fires when this scene loses focus (another scene pushed on top, or tab hidden)
		audio.music.pause

	onResume:
		// Fires when this scene regains focus
		audio.music.resume
```

---

## 3. Asset Preloading (`assets:`)

The `assets:` block declares all media the scene uses. It is processed before `init:` runs. The runtime fetches and decodes all assets before the first frame is drawn — eliminating pop-in.

Assets are declared once and referenced by name everywhere in the scene.

### Asset Types

#### `image`

Loads a static image for use with `canvas.image`.

```clean
assets:
	image "background" src="assets/bg.jpg"
	image "logo" src="assets/logo.png"
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `src` | string | yes | Path relative to `public/` folder |

#### `spritesheet`

Loads a tiled sprite sheet for use with `canvas.sprite`.

```clean
assets:
	spritesheet "hero" src="sprites/hero.png" frameWidth=48 frameHeight=64
	spritesheet "tiles" src="sprites/tiles.png" frameWidth=32 frameHeight=32
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `src` | string | yes | Path relative to `public/` folder |
| `frameWidth` | integer | yes | Width of one frame in pixels |
| `frameHeight` | integer | yes | Height of one frame in pixels |

Frames are indexed left-to-right, top-to-bottom starting at 0.

#### `sound`

Loads a short audio clip for use with `audio.play`. Buffered in memory — ideal for sound effects.

```clean
assets:
	sound "jump" src="sounds/jump.wav"
	sound "hit" src="sounds/hit.wav"
	sound "collect" src="sounds/coin.wav"
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `src` | string | yes | Path relative to `public/` folder |

#### `music`

Loads a streamed audio track for use with `audio.music.play`. Streamed from disk — ideal for background music.

```clean
assets:
	music "theme" src="music/theme.mp3"
	music "boss" src="music/boss-battle.mp3"
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `src` | string | yes | Path relative to `public/` folder |

#### `font`

Loads a web font for use with the `font=` attribute on `canvas.text`.

```clean
assets:
	font "mainFont" src="assets/fonts/roboto.woff2"
	font "titleFont" src="assets/fonts/bebas.woff2"
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `src` | string | yes | Path relative to `public/` folder |

### Complete Assets Example

```clean
canvasScene: width=800 height=600

	assets:
		image "background" src="assets/space-bg.jpg"
		image "logo" src="assets/logo.png"
		spritesheet "hero" src="sprites/hero.png" frameWidth=48 frameHeight=64
		spritesheet "enemy" src="sprites/enemy.png" frameWidth=32 frameHeight=32
		spritesheet "explosions" src="sprites/fx.png" frameWidth=64 frameHeight=64
		sound "shoot" src="sounds/laser.wav"
		sound "explode" src="sounds/explosion.wav"
		sound "powerup" src="sounds/powerup.wav"
		music "theme" src="music/space-theme.mp3"
		music "boss" src="music/boss.mp3"
		font "mainFont" src="assets/fonts/roboto.woff2"

	init:
		audio.music.play "theme" loop=true volume=0.5

	draw:
		canvas.image src="background" x=0 y=0 width=800 height=600
```

---

## 4. Persistent State (`state:`)

The `state:` block declares mutable variables that survive between frames. These are the scene's model — event handlers and `onFrame:` modify them; `draw:` reads them.

### Supported Types

| Type | Example | Description |
|------|---------|-------------|
| `number` | `number ballX = 400.0` | Floating-point value |
| `integer` | `integer score = 0` | Integer value |
| `string` | `string message = "Ready"` | Text value |
| `boolean` | `boolean paused = false` | True/false flag |

### Rules

- All state variables must have an initial value
- State variables are accessed directly by name — no `this.` prefix
- Variables declared in `state:` are visible in all sub-blocks of the same `canvasScene:`
- Variables are initialized once at scene load, before `init:` runs

### Example

```clean
canvasScene: width=800 height=600

	state:
		number ballX = 400.0
		number ballY = 300.0
		number ballVX = 200.0
		number ballVY = 150.0
		number ballRadius = 20.0
		integer score = 0
		integer lives = 3
		boolean paused = false
		boolean gameOver = false
		string phase = "play"

	onFrame: param="dt"
		if paused
			return
		ballX = ballX + ballVX * dt
		ballY = ballY + ballVY * dt
		if ballX < ballRadius
			ballVX = math.abs(ballVX)
		if ballX > 800.0 - ballRadius
			ballVX = 0.0 - math.abs(ballVX)
		if ballY < ballRadius
			ballVY = math.abs(ballVY)
		if ballY > 600.0 - ballRadius
			ballVY = 0.0 - math.abs(ballVY)

	draw:
		canvas.clear color="#1a1a2e"
		canvas.circle x=ballX y=ballY radius=ballRadius color="#e94560"
		canvas.text value="Score: " + score.toString() x=10 y=30 size=18 color="white"
```

---

## 5. One-Time Initialization (`init:`)

The `init:` block runs exactly once: after all assets have loaded, before the first frame is drawn. Use it to set initial state, start timelines, play background music, and register starting animations.

### Rules

- Runs after `assets:` are loaded; asset names are valid inside `init:`
- Runs before `draw:` and `onFrame:` begin
- Can play timelines, tweens, audio, and start state machines
- Cannot draw to the canvas (draw commands are only valid in `draw:`)

### Example 1 — Start music and timeline

```clean
canvasScene: width=800 height=600

	assets:
		music "theme" src="music/theme.mp3"

	state:
		number logoX = -200.0
		number logoAlpha = 0.0

	tween "logoIn":
		logoX from=-200 to=400
		duration = 1.2
		ease = "elasticOut"

	init:
		audio.music.play "theme" loop=true volume=0.6
		play tween "logoIn"

	draw:
		canvas.clear color="#0f0e17"
		canvas.alpha value=logoAlpha:
			canvas.text value="FRAME CANVAS" x=logoX y=300 size=48 color="white"
```

### Example 2 — Start a state machine

```clean
canvasScene: width=400 height=400

	state:
		boolean jumping = false
		boolean grounded = true

	animState "playerFSM":
		state "idle":
			play animSprite "heroIdle"
		state "jump":
			play animSprite "heroJump"
			onComplete "fall"
		transitions:
			"idle" -> "jump" when jumping
		initial = "idle"

	init:
		start animState "playerFSM"

	draw:
		canvas.clear color="#222"
```

---

## 6. Drawing API

All drawing commands live inside the `draw:` block. They execute in order, top to bottom, every frame. The canvas is an immediate-mode renderer — nothing persists between frames.

### 6.1 Clear

Clear the entire canvas. Always call this first in `draw:` unless you intentionally want motion trails.

```clean
canvas.clear
canvas.clear color="#1a1a2e"
canvas.clear color="white"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `color` | string | "black" | Background fill color |

### 6.2 Circle

```clean
canvas.circle x=400 y=300 radius=50 color="blue"
canvas.circle x=400 y=300 radius=50 color="red" filled=false
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 0 | Center X position |
| `y` | number | 0 | Center Y position |
| `radius` | number | 10 | Circle radius in pixels |
| `color` | string | "black" | Fill or stroke color |
| `filled` | boolean | true | `true` = filled; `false` = outline only |

### 6.3 Rectangle

```clean
canvas.rect x=100 y=100 width=200 height=120 color="#3b82f6"
canvas.rect x=100 y=100 width=200 height=120 color="white" filled=false
canvas.rect x=100 y=100 width=200 height=120 color="#3b82f6" cornerRadius=12
canvas.rect x=0 y=0 width=800 height=600 gradient="bg"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 0 | Top-left X position |
| `y` | number | 0 | Top-left Y position |
| `width` | number | 100 | Rectangle width |
| `height` | number | 100 | Rectangle height |
| `color` | string | "black" | Fill or stroke color |
| `gradient` | string | — | Named gradient to use as fill (overrides `color`) |
| `filled` | boolean | true | `true` = filled; `false` = outline only |
| `cornerRadius` | number | 0 | Rounded corner radius |

### 6.4 Line

```clean
canvas.line fromX=50 fromY=50 toX=350 toY=250 stroke=2 color="white"
canvas.line fromX=0 fromY=300 toX=800 toY=300 stroke=1 color="rgba(255,255,255,0.2)"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `fromX` | number | 0 | Start X |
| `fromY` | number | 0 | Start Y |
| `toX` | number | 100 | End X |
| `toY` | number | 100 | End Y |
| `stroke` | number | 1 | Line width in pixels |
| `color` | string | "black" | Line color |

### 6.5 Triangle

```clean
canvas.triangle x1=200 y1=100 x2=100 y2=300 x3=300 y3=300 color="#ff6b6b"
canvas.triangle x1=200 y1=100 x2=100 y2=300 x3=300 y3=300 color="white" filled=false
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x1` | number | 0 | First vertex X |
| `y1` | number | 0 | First vertex Y |
| `x2` | number | 0 | Second vertex X |
| `y2` | number | 0 | Second vertex Y |
| `x3` | number | 0 | Third vertex X |
| `y3` | number | 0 | Third vertex Y |
| `color` | string | "black" | Fill or stroke color |
| `filled` | boolean | true | `true` = filled; `false` = outline |

### 6.6 Ellipse

```clean
canvas.ellipse x=400 y=300 radiusX=150 radiusY=80 color="#feca57"
canvas.ellipse x=400 y=300 radiusX=150 radiusY=80 color="white" filled=false
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 0 | Center X |
| `y` | number | 0 | Center Y |
| `radiusX` | number | 50 | Horizontal radius |
| `radiusY` | number | 30 | Vertical radius |
| `color` | string | "black" | Fill or stroke color |
| `filled` | boolean | true | `true` = filled; `false` = outline |

### 6.7 Text

```clean
canvas.text value="Hello World" x=50 y=100 size=24 color="white"
canvas.text value="Score: " + score.toString() x=10 y=30 size=18 color="#feca57"
canvas.text value="GAME OVER" x=400 y=300 size=48 color="red" align="center"
canvas.text value="subtitle" x=400 y=360 size=16 color="gray" align="center" baseline="top"
canvas.text value="Hello" x=200 y=100 font="mainFont" size=24 color="white"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | string | "" | Text to render |
| `x` | number | 0 | X position |
| `y` | number | 0 | Y position |
| `size` | number | 16 | Font size in pixels |
| `color` | string | "black" | Text color |
| `font` | string | "sans-serif" | Asset name from `assets:` font declaration, or CSS font-family string |
| `align` | string | "left" | Horizontal alignment: `"left"`, `"center"`, `"right"` |
| `baseline` | string | "alphabetic" | Vertical baseline: `"top"`, `"middle"`, `"alphabetic"`, `"bottom"` |

### 6.8 Image

Draws a preloaded image (declared in `assets:`). The `src` attribute references the asset name, not the file path.

```clean
canvas.image src="background" x=0 y=0 width=800 height=600
canvas.image src="logo" x=300 y=200 width=200 height=100 opacity=0.8
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `src` | string | — | Asset name from `assets:` |
| `x` | number | 0 | Top-left X |
| `y` | number | 0 | Top-left Y |
| `width` | number | — | Display width (required) |
| `height` | number | — | Display height (required) |
| `opacity` | number | 1.0 | Transparency: 0.0 = invisible, 1.0 = opaque |

### 6.9 Sprite

Draws a sprite from a preloaded spritesheet. Supports static frames, animState-driven clips, and explicit sheet+clip combinations. All drawing modes use the unified `canvas.sprite` command.

```clean
// Static frame from sheet
canvas.sprite "hero" x=100 y=200 clip=3

// Driven by animState machine
canvas.sprite "hero" x=100 y=200 state="walk"

// Explicit sheet and clip
canvas.sprite "hero" x=100 y=200 sheet="run" clip=2

// With transform options
canvas.sprite "hero" x=100 y=200 state="walk" scaleX=2.0 scaleY=2.0 flipX=true
canvas.sprite "hero" x=100 y=200 clip=4 tint="#FF4400" tintStrength=0.5 alpha=0.8
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| (name) | string | — | Asset name from `assets:` spritesheet |
| `x` | number | 0 | Top-left X position |
| `y` | number | 0 | Top-left Y position |
| `clip` | integer | — | Frame index (0-based). Used for static or explicit-sheet mode |
| `state` | string | — | animState machine name. Drives frame automatically |
| `sheet` | string | — | Spritesheet asset name for explicit sheet+clip mode |
| `scaleX` | number | 1.0 | Horizontal scale factor |
| `scaleY` | number | 1.0 | Vertical scale factor |
| `flipX` | boolean | false | Mirror horizontally |
| `flipY` | boolean | false | Mirror vertically |
| `rotation` | number | 0 | Rotation in degrees |
| `alpha` | number | 1.0 | Opacity: 0.0 = invisible, 1.0 = opaque |
| `tint` | string | — | Tint color as hex string, e.g. `"#FF4400"` |
| `tintStrength` | number | 1.0 | Tint blend strength: 0.0 = no tint, 1.0 = full tint |
| `anchorX` | number | 0.0 | Horizontal anchor point (0.0 = left, 0.5 = center, 1.0 = right) |
| `anchorY` | number | 0.0 | Vertical anchor point (0.0 = top, 0.5 = center, 1.0 = bottom) |

### 6.10 Custom Paths (immediate mode)

For complex shapes not expressible with primitives, use the path API:

```clean
draw:
	canvas.beginPath
	canvas.moveTo x=50 y=150
	canvas.lineTo x=200 y=50
	canvas.lineTo x=350 y=150
	canvas.curveTo cpX=350 cpY=280 x=200 y=300
	canvas.curveTo cpX=50 cpY=280 x=50 y=150
	canvas.closePath
	canvas.fill color="#3b82f6"
	canvas.stroke color="white" width=2
```

| Command | Attributes | Description |
|---------|-----------|-------------|
| `canvas.beginPath` | — | Start a new path |
| `canvas.moveTo` | `x`, `y` | Move pen without drawing |
| `canvas.lineTo` | `x`, `y` | Line to point |
| `canvas.curveTo` | `cpX`, `cpY`, `x`, `y` | Quadratic bezier curve |
| `canvas.cubicTo` | `cp1x`, `cp1y`, `cp2x`, `cp2y`, `x`, `y` | Cubic bezier curve |
| `canvas.arc` | `x`, `y`, `radius`, `startAngle`, `endAngle` | Arc segment |
| `canvas.closePath` | — | Close path back to start |
| `canvas.fill` | `color` | Fill the path |
| `canvas.stroke` | `color`, `width` | Stroke the path |

### 6.11 Transform Groups

`canvas.group` applies a shared transform to a set of draw calls without manual save/restore. All children are drawn relative to the group's origin. Group attributes can be animated with tweens.

```clean
canvas.group x=400 y=300 rotation=0 scaleX=1.0 scaleY=1.0 alpha=1.0:
	canvas.sprite "eye1" x=-20 y=0 clip=0
	canvas.sprite "eye2" x=20 y=0 clip=0
	canvas.circle x=0 y=30 radius=15 color="red"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 0 | Group origin X |
| `y` | number | 0 | Group origin Y |
| `rotation` | number | 0 | Rotation in degrees |
| `scaleX` | number | 1.0 | Horizontal scale |
| `scaleY` | number | 1.0 | Vertical scale |
| `alpha` | number | 1.0 | Group opacity |

Groups are referenced by name in tweens using the state variable that drives their transform attribute:

```clean
state:
	number eyeGroupRotation = 0.0

tween "spin":
	eyeGroupRotation from=0 to=360
	duration = 2.0
	ease = "linear"
	repeat = -1

draw:
	canvas.group x=400 y=300 rotation=eyeGroupRotation:
		canvas.sprite "eye1" x=-20 y=0 clip=0
		canvas.sprite "eye2" x=20 y=0 clip=0
```

### 6.12 Clip / Mask Regions

`canvas.clip` restricts drawing to a specified region. Everything inside the `canvas.clip:` body is masked to that region.

```clean
// Rectangular clip
canvas.clip x=100 y=100 width=200 height=150:
	canvas.sprite "avatar" x=100 y=100 clip=0

// Circular clip
canvas.clip x=200 y=200 radius=80:
	canvas.sprite "portrait" x=120 y=120 clip=0

// Path-based clip
canvas.clip path="maskShape":
	canvas.sprite "content" x=0 y=0 clip=0
```

| Attribute | Type | Applies to | Description |
|-----------|------|------------|-------------|
| `x` | number | rect, circle | Origin X |
| `y` | number | rect, circle | Origin Y |
| `width` | number | rect | Clip rectangle width |
| `height` | number | rect | Clip rectangle height |
| `radius` | number | circle | Circular clip radius |
| `path` | string | path | Named `path` to use as clip shape |

### Drawing Complete Example

```clean
canvasScene: width=600 height=400

	state:
		number pulse = 0.0
		number pulseDir = 1.0

	onFrame: param="dt"
		pulse = pulse + pulseDir * dt * 2.0
		if pulse > 1.0
			pulseDir = 0.0 - 1.0
		if pulse < 0.0
			pulseDir = 1.0

	draw:
		canvas.clear color="#0f0e17"

		// Background grid lines
		canvas.line fromX=0 fromY=200 toX=600 toY=200 stroke=1 color="rgba(255,255,255,0.05)"
		canvas.line fromX=300 fromY=0 toX=300 toY=400 stroke=1 color="rgba(255,255,255,0.05)"

		// Pulsing center circle
		number r = 60.0 + pulse * 20.0
		canvas.circle x=300 y=200 radius=r color="#3b82f6"
		canvas.circle x=300 y=200 radius=r color="white" filled=false

		// Corner triangles
		canvas.triangle x1=0 y1=0 x2=60 y2=0 x3=0 y3=60 color="#e94560"
		canvas.triangle x1=600 y1=0 x2=540 y2=0 x3=600 y3=60 color="#e94560"

		// Ellipse orbiting
		canvas.save
		canvas.translate x=300 y=200
		canvas.rotate angle=pulse * 180.0
		canvas.ellipse x=120 y=0 radiusX=20 radiusY=10 color="#feca57"
		canvas.restore

		// Label
		canvas.text value="FRAME CANVAS" x=300 y=380 size=14 color="rgba(255,255,255,0.4)" align="center"
```

---

## 7. Gradients

Named gradient definitions can be used anywhere a color value is accepted. Define them once at the scene level; reference by name using the `gradient=` attribute on drawing commands.

### Linear Gradient

```clean
gradient "sunset" linear x1=0 y1=0 x2=800 y2=0:
	stop 0.0 color="#ff6b6b"
	stop 0.5 color="#feca57"
	stop 1.0 color="#48dbfb"

gradient "sky" linear x1=0 y1=0 x2=0 y2=600:
	stop 0.0 color="#1e3a5f"
	stop 1.0 color="#0f0e17"
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `x1`, `y1` | number | yes | Gradient start point |
| `x2`, `y2` | number | yes | Gradient end point |

### Radial Gradient

```clean
gradient "radialGlow" radial cx=400 cy=300 r=200:
	stop 0.0 color="#ffffff"
	stop 0.4 color="#3b82f6"
	stop 1.0 color="transparent"

gradient "spotlight" radial cx=400 cy=600 r=400:
	stop 0.0 color="rgba(255,255,200,0.3)"
	stop 1.0 color="transparent"
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `cx`, `cy` | number | yes | Center of gradient |
| `r` | number | yes | Outer radius |

### Stop Syntax

Each stop is: `stop <position> color=<value>` where position is 0.0 (start) to 1.0 (end).

### Using Gradients in Draw

Reference a named gradient with the `gradient=` attribute on any shape command that accepts a fill:

```clean
canvasScene: width=800 height=600

	gradient "bg" linear x1=0 y1=0 x2=0 y2=600:
		stop 0.0 color="#1e3a5f"
		stop 1.0 color="#0f0e17"

	gradient "orb" radial cx=400 cy=300 r=150:
		stop 0.0 color="#a78bfa"
		stop 0.6 color="#3b82f6"
		stop 1.0 color="transparent"

	gradient "bar" linear x1=0 y1=0 x2=600 y2=0:
		stop 0.0 color="#ff6b6b"
		stop 1.0 color="#48dbfb"

	draw:
		canvas.rect x=0 y=0 width=800 height=600 gradient="bg"
		canvas.circle x=400 y=300 radius=150 gradient="orb"
		canvas.rect x=100 y=500 width=600 height=20 gradient="bar" cornerRadius=10
```

---

## 8. Effects & Filters

Effect blocks wrap drawing commands and apply visual post-processing. They form a composable stack — effects can be nested.

### `canvas.shadow`

Applies a drop shadow to all drawing inside the block.

```clean
canvas.shadow blur=8 offsetX=4 offsetY=4 color="rgba(0,0,0,0.4)":
	canvas.rect x=100 y=100 width=200 height=120 color="white"
	canvas.text value="Drop Shadow" x=200 y=165 size=18 color="black" align="center"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `blur` | number | 4 | Shadow blur radius |
| `offsetX` | number | 0 | Horizontal shadow offset |
| `offsetY` | number | 4 | Vertical shadow offset |
| `color` | string | "rgba(0,0,0,0.5)" | Shadow color |

### `canvas.blur`

Applies a Gaussian blur to all drawing inside the block.

```clean
canvas.blur radius=10:
	canvas.image src="background" x=0 y=0 width=800 height=600
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `radius` | number | 4 | Blur radius in pixels |

### `canvas.glow`

Applies a colored glow effect.

```clean
canvas.glow color="#00ff88" blur=20:
	canvas.circle x=400 y=300 radius=50 color="#00ff88"

canvas.glow color="#ff6b6b" blur=15:
	canvas.text value="HOT" x=400 y=200 size=64 color="white" align="center"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `color` | string | "white" | Glow color |
| `blur` | number | 10 | Glow spread radius |

### `canvas.alpha`

Applies uniform transparency to all drawing inside the block.

```clean
canvas.alpha value=0.5:
	canvas.rect x=0 y=0 width=800 height=600 color="black"

canvas.alpha value=fadeIn:
	canvas.text value="LOADING" x=400 y=300 size=32 color="white" align="center"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | number | 1.0 | Opacity: 0.0 = invisible, 1.0 = fully opaque |

### `canvas.blendMode`

Sets the compositing blend mode for drawing inside the block.

```clean
canvas.blendMode mode="add":
	canvas.circle x=400 y=300 radius=100 color="#ff4444"
	canvas.circle x=440 y=300 radius=100 color="#4444ff"

canvas.blendMode mode="multiply":
	canvas.image src="overlay" x=0 y=0 width=800 height=600
```

| `mode` Value | Description |
|-------------|-------------|
| `"normal"` | Standard alpha compositing (default) |
| `"multiply"` | Darkening blend — good for shadows |
| `"screen"` | Brightening blend — good for highlights |
| `"overlay"` | Contrast-enhancing blend |
| `"add"` | Additive blend — good for fire, lasers, magic |
| `"difference"` | Color inversion at overlap |

### Nested Effects Example

```clean
draw:
	canvas.clear color="#0f0e17"

	// Background with blur
	canvas.blur radius=6:
		canvas.image src="background" x=0 y=0 width=800 height=600

	// Glowing UI element with shadow
	canvas.shadow blur=12 offsetX=0 offsetY=0 color="#3b82f6":
		canvas.glow color="#3b82f6" blur=20:
			canvas.rect x=200 y=200 width=400 height=200 color="#1e3a5f" cornerRadius=16
			canvas.rect x=200 y=200 width=400 height=200 color="#3b82f6" filled=false cornerRadius=16

	// Particle layer with additive blend
	canvas.blendMode mode="add":
		canvas.alpha value=0.8:
			canvas.circle x=400 y=300 radius=30 color="white"
```

---

## 9. Transforms

Transforms affect all drawing commands that follow until `canvas.restore` is called. They compose multiplicatively — a translate followed by a rotate rotates around the translated origin.

### Save / Restore Stack

```clean
canvas.save      // push current transform onto stack
// ... apply transforms and draw ...
canvas.restore   // pop back to saved transform
```

Always pair `canvas.save` with `canvas.restore`. Forgetting `canvas.restore` causes transform state to leak into subsequent draw calls.

### Translate

Moves the origin by (x, y). All subsequent positions are relative to the new origin.

```clean
canvas.translate x=400 y=300
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 0 | Horizontal offset |
| `y` | number | 0 | Vertical offset |

### Rotate

Rotates around the current origin. Use `canvas.translate` first to set the rotation pivot.

```clean
canvas.rotate angle=45
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `angle` | number | 0 | Rotation in degrees (clockwise) |

### Scale

Scales from the current origin.

```clean
canvas.scale x=2.0 y=2.0
canvas.scale x=1.0 y=-1.0   // flip vertically
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number | 1 | Horizontal scale factor |
| `y` | number | 1 | Vertical scale factor |

### Compound Transform Example

```clean
canvasScene: width=600 height=400

	state:
		number angle = 0.0
		number moonAngle = 0.0

	onFrame: param="dt"
		angle = angle + 45.0 * dt
		moonAngle = moonAngle + 120.0 * dt

	draw:
		canvas.clear color="#0f0e17"

		// Planet with orbiting moon
		canvas.save
		canvas.translate x=300 y=200
		canvas.rotate angle=angle

		// Planet
		canvas.circle x=0 y=0 radius=40 color="#3b82f6"

		// Moon orbit (nested transform)
		canvas.save
		canvas.rotate angle=moonAngle
		canvas.circle x=80 y=0 radius=12 color="#94a3b8"
		canvas.restore

		canvas.restore

		// Stars (fixed, not affected by planet transform)
		canvas.circle x=50 y=30 radius=2 color="white"
		canvas.circle x=520 y=80 radius=1 color="white"
		canvas.circle x=180 y=350 radius=2 color="white"
```

---

## 10. Layers

Named render layers solve the z-ordering problem cleanly. Instead of manually sorting draw calls, assign them to a layer. The runtime draws all layers in ascending `z` order, regardless of the order they appear in code.

### Declaring Layers

```clean
layers:
	layer "background" z=0
	layer "terrain" z=5
	layer "world" z=10
	layer "characters" z=20
	layer "projectiles" z=25
	layer "effects" z=30
	layer "ui-bg" z=90
	layer "hud" z=100
```

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `z` | integer | yes | Draw order (lower = drawn first, appears behind) |

### Drawing to Layers

Use `on layer "name":` inside `draw:` to route draw commands to a specific layer.

```clean
draw:
	on layer "background":
		canvas.clear color="#0f0e17"
		canvas.image src="background" x=0 y=0 width=800 height=600

	on layer "world":
		canvas.rect x=0 y=500 width=800 height=100 color="#2d6a4f"

	on layer "characters":
		canvas.circle x=playerX y=playerY radius=20 color="#3b82f6"
		canvas.circle x=enemyX y=enemyY radius=18 color="#e94560"

	on layer "effects":
		canvas.glow color="#feca57" blur=15:
			canvas.circle x=coinX y=coinY radius=8 color="#feca57"

	on layer "hud":
		canvas.text value="Score: " + score.toString() x=10 y=30 size=20 color="white"
		canvas.text value="Lives: " + lives.toString() x=700 y=30 size=20 color="white" align="right"
```

### Rules

- All `on layer` blocks accumulate draw calls into that layer's buffer each frame
- Layers without any draw calls are skipped
- Drawing outside an `on layer` block goes to the default (z=0) layer
- Layer names are case-sensitive

---

## 11. Event Handlers

Event handlers respond to user input. Each handler block names the injected variable(s) via `param=`.

### `onFrame:`

Fires every animation frame. This is the game loop — use it to update physics, check collisions, advance animation state, and modify state variables.

```clean
onFrame: param="dt"
	// dt is delta time in seconds (typically ~0.016 at 60fps)
	ballX = ballX + velocityX * dt
	ballY = ballY + velocityY * dt
```

| `param` value | Variable name | Type | Description |
|--------------|--------------|------|-------------|
| `"dt"` (default) | `dt` | number | Seconds since last frame |

### `onPointerDown:`

Fires when the user presses the mouse button or starts a touch.

```clean
onPointerDown: param="x,y"
	// x and y are canvas-relative coordinates
	if collision.pointRect px=x py=y rx=btnX ry=btnY rw=120 rh=40 == 1
		scene.change "GameScene"
```

| `param` value | Variables | Type | Description |
|--------------|-----------|------|-------------|
| `"x,y"` (default) | `x`, `y` | number | Canvas-relative pointer position |

### `onPointerMove:`

Fires every time the pointer moves over the canvas.

```clean
onPointerMove: param="mx,my"
	cursorX = mx
	cursorY = my
	if collision.pointRect px=mx py=my rx=100 ry=100 rw=200 rh=50 == 1
		hovering = true
	if collision.pointRect px=mx py=my rx=100 ry=100 rw=200 rh=50 == 0
		hovering = false
```

### `onPointerUp:`

Fires when the mouse button is released or a touch ends.

```clean
onPointerUp: param="x,y"
	if dragging
		dragging = false
		targetX = x
		targetY = y
```

### `onKeyDown:`

Fires when a keyboard key is pressed. The `key` variable contains the key name as a string.

```clean
onKeyDown: param="key"
	if key == "ArrowLeft"
		moveLeft = true
	if key == "ArrowRight"
		moveRight = true
	if key == "Space"
		if grounded
			velocityY = jumpForce
			grounded = false
	if key == "Escape"
		scene.push "PauseMenu"
	if key == "r"
		scene.change "GameScene"
```

Common key name strings: `"ArrowUp"`, `"ArrowDown"`, `"ArrowLeft"`, `"ArrowRight"`, `"Space"`, `"Enter"`, `"Escape"`, `"Backspace"`, `"a"`..`"z"`, `"0"`..`"9"`, `"F1"`..`"F12"`.

### `onKeyUp:`

Fires when a keyboard key is released.

```clean
onKeyUp: param="key"
	if key == "ArrowLeft"
		moveLeft = false
	if key == "ArrowRight"
		moveRight = false
```

### Custom Parameter Names

The `param` attribute value is the variable name you want injected:

```clean
onPointerDown: param="clickX,clickY"
	// clickX and clickY are available here

onKeyDown: param="pressedKey"
	// pressedKey is available here

onFrame: param="deltaTime"
	// deltaTime is available here
```

### Combined Input Example

```clean
canvasScene: width=800 height=600

	state:
		number playerX = 400.0
		number playerY = 300.0
		number playerSpeed = 200.0
		boolean moveLeft = false
		boolean moveRight = false
		boolean moveUp = false
		boolean moveDown = false
		number cursorX = 0.0
		number cursorY = 0.0

	onKeyDown: param="key"
		if key == "ArrowLeft"
			moveLeft = true
		if key == "ArrowRight"
			moveRight = true
		if key == "ArrowUp"
			moveUp = true
		if key == "ArrowDown"
			moveDown = true

	onKeyUp: param="key"
		if key == "ArrowLeft"
			moveLeft = false
		if key == "ArrowRight"
			moveRight = false
		if key == "ArrowUp"
			moveUp = false
		if key == "ArrowDown"
			moveDown = false

	onPointerMove: param="mx,my"
		cursorX = mx
		cursorY = my

	onFrame: param="dt"
		if moveLeft
			playerX = playerX - playerSpeed * dt
		if moveRight
			playerX = playerX + playerSpeed * dt
		if moveUp
			playerY = playerY - playerSpeed * dt
		if moveDown
			playerY = playerY + playerSpeed * dt

	draw:
		canvas.clear color="#1a1a2e"
		canvas.line fromX=playerX fromY=playerY toX=cursorX toY=cursorY stroke=1 color="rgba(255,255,255,0.3)"
		canvas.circle x=playerX y=playerY radius=20 color="#3b82f6"
		canvas.circle x=cursorX y=cursorY radius=4 color="white"
```

---

## 12. Tween System

Tweens animate a state variable from one value to another over time. Tweens can be named and triggered on demand, or fired inline with `animate`.

### Named Tween Definition

Define a tween at scene level. It does not play until triggered. The variable being animated is declared on its own line with `from=` and `to=` as flat attributes. Tween-level config attributes use `=` syntax.

```clean
tween "ballSlide":
	ballX from=0 to=700
	duration = 1.5
	ease = "elasticOut"
	delay = 0.3
	repeat = 0
	yoyo = false
```

### Tween Attributes

**Variable line** (one per animated variable):

| Field | Description |
|-------|-------------|
| `<varName>` | State variable to animate (first token on the line) |
| `from=` | Starting value. Omit to start from current value |
| `to=` | Target value (required) |

**Config attributes** (one per line, `key = value` format):

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `duration` | number | 1.0 | Animation duration in seconds |
| `ease` | string | `"linear"` | Easing name (see Section 21) |
| `delay` | number | 0.0 | Seconds to wait before starting |
| `repeat` | integer | 0 | `0`=play once, `-1`=loop forever, `N`=play N+1 times |
| `yoyo` | boolean | false | Reverse direction on each repeat |
| `onComplete` | string | — | State variable name to set to `true` when done |
| `along` | string | — | Named path to follow (see Section 15) |
| `orient` | boolean | false | Auto-rotate companion angle variable when following a path |

### Multiple Variables in One Tween

Animate several variables simultaneously with synchronized timing:

```clean
tween "intro":
	titleX from=0 to=400
	titleY from=-100 to=200
	titleAlpha from=0 to=1
	duration = 1.0
	ease = "cubicOut"
```

### Triggering Tweens

```clean
init:
	play tween "ballSlide"

onPointerDown: param="x,y"
	stop tween "ballSlide"
	play tween "ballSlide"

onKeyDown: param="key"
	if key == "Space"
		pause tween "ballSlide"
	if key == "r"
		stop tween "ballSlide"
		play tween "ballSlide"
```

### Inline Animate

For one-off animations that don't need a name, use `animate` directly in any handler:

```clean
onPointerDown: param="px,py"
	animate ballX to=px duration=0.4 ease="easeOut"
	animate ballY to=py duration=0.4 ease="easeOut"

onKeyDown: param="key"
	if key == "Space"
		animate playerY to=50 duration=0.3 ease="cubicOut"

// With from:
animate logoX from=-300 to=400 duration=1.0 ease="easeOutElastic"
```

`animate` inline attributes: `to` (required), `from` (optional), `duration`, `ease`, `delay`.

### Custom Easing — `customEase`

Define a custom cubic bezier easing at scene level. It is automatically registered under `ease = "name"` and can be used in any tween or `animate` call.

```clean
customEase "snap" x1=0.68 y1=-0.55 x2=0.265 y2=1.55

tween "popIn":
	btnScale from=0 to=1
	duration = 0.5
	ease = "snap"
```

| Attribute | Type | Description |
|-----------|------|-------------|
| `x1` | number | First control point X (0.0–1.0) |
| `y1` | number | First control point Y (can exceed 0–1 for overshoot) |
| `x2` | number | Second control point X (0.0–1.0) |
| `y2` | number | Second control point Y |

### Tween Lifecycle Example

```clean
canvasScene: width=800 height=400

	state:
		number logoX = -300.0
		number logoAlpha = 0.0
		number btnScale = 0.0

	tween "logoEnter":
		logoX from=-300 to=400
		duration = 0.9
		ease = "elasticOut"

	tween "fadeIn":
		logoAlpha from=0 to=1.0
		duration = 0.5
		ease = "easeIn"

	tween "btnPop":
		btnScale from=0 to=1.0
		duration = 0.6
		ease = "bounceOut"
		delay = 0.8

	init:
		play tween "logoEnter"
		play tween "fadeIn"
		play tween "btnPop"

	draw:
		canvas.clear color="#0f0e17"
		canvas.alpha value=logoAlpha:
			canvas.save
			canvas.translate x=logoX y=200
			canvas.text value="FRAME CANVAS" x=0 y=0 size=48 color="white" align="center"
			canvas.restore
		canvas.save
		canvas.translate x=400 y=320
		canvas.scale x=btnScale y=btnScale
		canvas.rect x=-80 y=-20 width=160 height=40 color="#3b82f6" cornerRadius=8
		canvas.text value="START" x=0 y=0 size=16 color="white" align="center" baseline="middle"
		canvas.restore
```

---

## 13. Timeline Sequencing

Timelines sequence tweens and actions in absolute and relative time, enabling precise multi-element choreography. Named timelines are defined at scene level and triggered by name.

### Timeline Syntax

```clean
timeline "introSequence":
	at 0.0:
		animate logoX to=400 duration=0.8 ease="easeOut"
	at 0.6:
		animate logoAlpha to=1 duration=0.5
	after:
		// Immediately after the previous block's animations end
		animate taglineY to=300 duration=0.6 ease="elasticOut"
	after overlap=0.2:
		// Start 0.2 seconds before the previous block ends
		animate taglineAlpha to=1 duration=0.4
	after gap=0.1:
		// Start 0.1 seconds after the previous block ends
		animate subAlpha to=1 duration=0.3
	together:
		// All start at the same absolute time as each other
		animate btn1Alpha to=1 duration=0.3
		animate btn2Alpha to=1 duration=0.3 delay=0.1
		animate btn3Alpha to=1 duration=0.3 delay=0.2
```

### Time Position Keywords

| Keyword | Meaning |
|---------|---------|
| `at <seconds>:` | Absolute time from timeline start |
| `after:` | Immediately after the previous block's last animation finishes |
| `after gap=<n>:` | `n` seconds after the previous block ends (forward gap) |
| `after overlap=<n>:` | Start `n` seconds before the previous block ends |
| `together:` | All animations start simultaneously at the same time as each other |
| `together stagger=<n>:` | Each child starts `n` seconds after the previous child |

### Timeline Stagger

`stagger=` on a `together:` block delays each child animation by `stagger` seconds more than the previous one:

```clean
timeline "entrance":
	together stagger=0.08:
		animate card1Alpha to=1 duration=0.3
		animate card2Alpha to=1 duration=0.3
		animate card3Alpha to=1 duration=0.3
		animate card4Alpha to=1 duration=0.3
```

card1 starts at t=0, card2 at t=0.08, card3 at t=0.16, card4 at t=0.24.

### Triggering Timelines

```clean
init:
	play timeline "introSequence"

onKeyDown: param="key"
	if key == "Space"
		pause timeline "introSequence"
	if key == "r"
		stop timeline "introSequence"
		play timeline "introSequence"
	if key == "f"
		seek timeline "introSequence" to=2.0
```

### Timeline Callbacks

```clean
timeline "battle":
	at 0.0:
		animate flashAlpha to=1 duration=0.1
	at 0.1:
		animate flashAlpha to=0 duration=0.3
	after:
		animate shakeX to=5 duration=0.05 repeat=6 yoyo=true
	onComplete:
		battleStarted = true
```

`onComplete:` sets a boolean state variable to `true` when the timeline finishes.

### Complete Timeline Example

```clean
canvasScene: width=800 height=500

	state:
		number logoX = -400.0
		number logoAlpha = 0.0
		number subAlpha = 0.0
		number btn1Alpha = 0.0
		number btn2Alpha = 0.0
		number dividerW = 0.0
		boolean ready = false

	timeline "mainMenu":
		at 0.0:
			animate logoX to=400 duration=0.9 ease="elasticOut"
			animate logoAlpha to=1.0 duration=0.4 ease="easeIn"
		after:
			animate dividerW to=600 duration=0.5 ease="cubicOut"
		after overlap=0.1:
			animate subAlpha to=1.0 duration=0.5
		after gap=0.1:
			animate btn1Alpha to=1.0 duration=0.3 ease="easeOut"
		after overlap=0.1:
			animate btn2Alpha to=1.0 duration=0.3 ease="easeOut"
		onComplete:
			ready = true

	init:
		play timeline "mainMenu"

	draw:
		canvas.clear color="#0f0e17"
		canvas.alpha value=logoAlpha:
			canvas.text value="SPACE VOYAGE" x=logoX y=160 size=52 color="white" align="center"
		canvas.rect x=(400.0 - dividerW * 0.5) y=220 width=dividerW height=2 color="#3b82f6"
		canvas.alpha value=subAlpha:
			canvas.text value="A Frame Canvas Demo" x=400 y=260 size=18 color="#94a3b8" align="center"
		canvas.alpha value=btn1Alpha:
			canvas.rect x=250 y=310 width=140 height=44 color="#3b82f6" cornerRadius=8
			canvas.text value="NEW GAME" x=320 y=332 size=14 color="white" align="center"
		canvas.alpha value=btn2Alpha:
			canvas.rect x=410 y=310 width=140 height=44 color="#1e293b" cornerRadius=8
			canvas.text value="OPTIONS" x=480 y=332 size=14 color="#94a3b8" align="center"
```

---

## 14. Animation State Machine (`animState`)

Animation state machines describe how a character or object transitions between named animation states. Transitions are driven by boolean expressions on state variables, evaluated every frame.

### State Machine Syntax

```clean
animState "playerFSM":
	state "idle":
		play animSprite "heroIdle"

	state "walk":
		play animSprite "heroWalk"

	state "jump":
		play animSprite "heroJump"
		onComplete "fall"    // transition when current animSprite completes a full cycle

	state "fall":
		play animSprite "heroFall"

	state "land":
		play animSprite "heroLand"
		onComplete "idle"

	state "hurt":
		play animSprite "heroHurt"
		onComplete "idle"

	transitions:
		"idle" -> "walk" when speedX > 20
		"walk" -> "idle" when speedX < 5
		"idle" -> "jump" when jumping
		"walk" -> "jump" when jumping
		"jump" -> "fall" when velocityY > 0
		"fall" -> "land" when grounded
		"walk" -> "hurt" when tookDamage
		"idle" -> "hurt" when tookDamage

	initial = "idle"
```

### State Machine Rules

- The state machine is started with `start animState "name"` in `init:`
- Transitions in the `transitions:` block are evaluated every frame in declaration order
- The first matching transition fires; only one transition fires per frame
- `onComplete "stateName"` fires when the active `animSprite` finishes a full cycle (`loop = false` sprites only)
- Transition conditions use `when` followed by a boolean expression
- State names are strings; they must match a declared `state "..."` block exactly
- `initial = "stateName"` sets the starting state

### Supported Transition Conditions

| Syntax | Meaning |
|--------|---------|
| `when variableName` | True when the boolean variable is `true` |
| `when variable > value` | Numeric comparison |
| `when variable < value` | Numeric comparison |
| `when variable == value` | Equality |
| `when variable != value` | Not equal |

### Triggering and Querying

```clean
init:
	start animState "playerFSM"

// Force a transition from code (overrides condition evaluation for one frame):
onPointerDown: param="x,y"
	animState.force "playerFSM" to="hurt"

// Query current state:
onFrame: param="dt"
	string currentState = animState.current "playerFSM"
	if currentState == "walk"
		footstepTimer = footstepTimer + dt
```

### Complete State Machine Example

```clean
canvasScene: width=600 height=400

	assets:
		spritesheet "hero" src="sprites/hero.png" frameWidth=48 frameHeight=64

	state:
		number playerX = 100.0
		number playerY = 300.0
		number velocityX = 0.0
		number velocityY = 0.0
		boolean jumping = false
		boolean grounded = true
		boolean moveRight = false
		boolean moveLeft = false
		number speedX = 0.0

	animSprite "heroIdle":
		sheet = "hero"
		frames = 0..3
		fps = 8
		loop = true

	animSprite "heroWalk":
		sheet = "hero"
		frames = 4..11
		fps = 14
		loop = true

	animSprite "heroJump":
		sheet = "hero"
		frames = 12..14
		fps = 10
		loop = false

	animSprite "heroFall":
		sheet = "hero"
		frames = 15..16
		fps = 8
		loop = true

	animSprite "heroLand":
		sheet = "hero"
		frames = 17..18
		fps = 12
		loop = false

	animState "playerFSM":
		state "idle":
			play animSprite "heroIdle"
		state "walk":
			play animSprite "heroWalk"
		state "jump":
			play animSprite "heroJump"
			onComplete "fall"
		state "fall":
			play animSprite "heroFall"
		state "land":
			play animSprite "heroLand"
			onComplete "idle"
		transitions:
			"idle" -> "walk" when speedX > 20
			"walk" -> "idle" when speedX < 5
			"idle" -> "jump" when jumping
			"walk" -> "jump" when jumping
			"jump" -> "fall" when velocityY > 50
			"fall" -> "land" when grounded
		initial = "idle"

	init:
		start animState "playerFSM"

	onKeyDown: param="key"
		if key == "ArrowRight"
			moveRight = true
		if key == "ArrowLeft"
			moveLeft = true
		if key == "Space"
			if grounded
				velocityY = -400.0
				jumping = true
				grounded = false

	onKeyUp: param="key"
		if key == "ArrowRight"
			moveRight = false
		if key == "ArrowLeft"
			moveLeft = false

	onFrame: param="dt"
		if moveRight
			velocityX = 150.0
		if moveLeft
			velocityX = -150.0
		if moveRight == false
			if moveLeft == false
				velocityX = 0.0
		speedX = math.abs(velocityX)
		velocityY = velocityY + 800.0 * dt
		playerX = playerX + velocityX * dt
		playerY = playerY + velocityY * dt
		if playerY > 300.0
			playerY = 300.0
			velocityY = 0.0
			grounded = true
			jumping = false

	draw:
		canvas.clear color="#87ceeb"
		canvas.rect x=0 y=340 width=600 height=60 color="#4a7c59"
		canvas.sprite "hero" x=playerX y=playerY state="playerFSM"
```

---

## 15. Path Animation

Animate objects along bezier curves defined with the `path` block.

### Defining a Path

```clean
path "figure8":
	moveTo x=100 y=300
	curveTo x=400 y=300 cp1x=100 cp1y=100 cp2x=400 cp2y=100
	curveTo x=700 y=300 cp1x=400 cp1y=500 cp2x=700 cp2y=500
	curveTo x=400 y=300 cp1x=700 cp1y=100 cp2x=400 cp2y=100
	curveTo x=100 y=300 cp1x=400 cp1y=500 cp2x=100 cp2y=500
	closed = true

path "arc":
	moveTo x=100 y=500
	curveTo x=700 y=500 cp1x=200 cp1y=100 cp2x=600 cp2y=100
```

### Path Command Attributes

| Command | Attributes | Description |
|---------|-----------|-------------|
| `moveTo` | `x`, `y` | Move to starting point |
| `lineTo` | `x`, `y` | Straight line segment |
| `curveTo` | `cp1x`, `cp1y`, `cp2x`, `cp2y`, `x`, `y` | Cubic bezier segment |
| `quadTo` | `cpX`, `cpY`, `x`, `y` | Quadratic bezier segment |
| `closed` | `= true/false` | Close path back to start |

### Animating Along a Path

```clean
tween "orbitShip":
	along = "figure8"
	shipX to=0
	shipY to=0
	duration = 4.0
	ease = "linear"
	orient = true         // auto-rotate shipAngle to face direction of travel
	repeat = -1

tween "ballArc":
	along = "arc"
	ballX to=0
	ballY to=0
	duration = 1.2
	ease = "cubicInOut"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `along` | string | — | Named path to follow |
| `orient` | boolean | false | Auto-rotate a companion angle variable |

When `along` is set, the X and Y variables receive their positions from the path. The `to=` on each variable line is ignored — path position drives the value.

### Drawing a Path as a Visible Line

```clean
draw:
	canvas.drawPath "figure8" stroke=2 color="rgba(255,255,255,0.2)"
	canvas.drawPath "arc" stroke=3 color="#3b82f6"
```

### Complete Path Animation Example

```clean
canvasScene: width=800 height=600

	state:
		number shipX = 100.0
		number shipY = 300.0
		number shipAngle = 0.0

	path "orbit":
		moveTo x=400 y=100
		curveTo x=400 y=500 cp1x=700 cp1y=100 cp2x=700 cp2y=500
		curveTo x=400 y=100 cp1x=100 cp1y=500 cp2x=100 cp2y=100
		closed = true

	tween "shipFlight":
		along = "orbit"
		shipX to=0
		shipY to=0
		duration = 5.0
		ease = "linear"
		orient = true
		repeat = -1

	init:
		play tween "shipFlight"

	draw:
		canvas.clear color="#0f0e17"
		canvas.drawPath "orbit" stroke=1 color="rgba(59,130,246,0.15)"
		canvas.save
		canvas.translate x=shipX y=shipY
		canvas.rotate angle=shipAngle
		canvas.triangle x1=0 y1=-10 x2=-6 y2=8 x3=6 y3=8 color="#feca57"
		canvas.restore
		canvas.glow color="#feca57" blur=12:
			canvas.circle x=shipX y=shipY radius=4 color="#feca57"
```

---

## 16. Particle Systems

Declarative particle emitters produce high-volume visual effects efficiently. All particle behavior is described in a `particles` block and triggered with `emit particles` or `start particles`.

### Emitter Declaration

```clean
particles "confetti":
	emitter:
		rate = 80            // particles spawned per second (continuous)
		burst = 0            // extra particles on trigger (one-shot spike)
		shape = "point"      // spawn area shape (see below)

	lifetime = 1.5..3.0      // seconds (random value in range)

	initial:
		speed = 150..400     // initial speed in pixels/sec
		angle = -140..-40    // launch direction in degrees (0=right, -90=up)
		spin = -180..180     // rotation speed in degrees/sec
		size = 6..14         // initial size in pixels
		color = ["#ff6b6b", "#feca57", "#48dbfb", "#ff9ff3", "#54a0ff"]

	overLifetime:
		size = 1.0 -> 0.2    // size multiplier at start -> end of lifetime
		alpha = 1.0 -> 0.0   // opacity at start -> end

	forces:
		gravity = 300        // pixels/sec² downward acceleration
		drag = 0.96          // velocity multiplier per frame (< 1 = friction)

	shape = "circle"         // render shape: "circle", "rect"
	// OR for image-based particles:
	// image = "sparkleGlow"
	// size = 12
```

### `emitter:` Sub-block

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `rate` | number | 0 | Particles spawned per second (continuous mode) |
| `burst` | integer | 0 | Extra particles spawned on `emit` trigger (one-shot spike) |
| `shape` | string | `"point"` | Spawn area shape (see Spawn Shape Values below) |

### Spawn Shape Values

| `shape` value | Description |
|--------------|-------------|
| `"point"` | All particles spawn at the emit position |
| `"circle radius=50"` | Spawn anywhere inside a circle of radius 50 |
| `"rect w=200 h=10"` | Spawn anywhere inside a rectangle |

### Render Attributes (top-level in `particles` block)

| Attribute | Type | Description |
|-----------|------|-------------|
| `shape` | string | Render shape: `"circle"` or `"rect"` |
| `image` | string | Asset name for image-based particles (overrides `shape`) |
| `size` | number | Particle size in pixels when using `image` |

### `overLifetime:` Sub-block

Applies a value ramp over each particle's lifetime using `start -> end` syntax:

| Attribute | Syntax | Description |
|-----------|--------|-------------|
| `size` | `1.0 -> 0.2` | Size multiplier: 1.0 at birth, 0.2 at death |
| `alpha` | `1.0 -> 0.0` | Opacity: 1.0 at birth, 0.0 at death |

### `forces:` Sub-block

| Attribute | Type | Description |
|-----------|------|-------------|
| `gravity` | number | Pixels/sec² downward acceleration |
| `drag` | number | Velocity multiplier per frame (< 1.0 = friction) |
| `wind x=<n> y=<n>` | — | Constant directional force |
| `turbulence strength=<n> scale=<n>` | — | Perlin noise force field |
| `attractor x=<n> y=<n> strength=<n>` | — | Point attractor / repeller (negative strength = repel) |

Forces example:

```clean
particles "leaves":
	emitter:
		rate = 5
	forces:
		wind x=0.5 y=0.0
		turbulence strength=0.3 scale=80.0
		attractor x=400 y=300 strength=0.1
```

### Triggering Emitters

```clean
// One-time burst (fires immediately, then stops)
emit particles "confetti" x=400 y=300

// Continuous emitter (keeps emitting until stopped)
start particles "confetti" x=400 y=300
stop particles "confetti"

// Dynamic position (emitter follows state variables)
emit particles "thrusterFlame" x=shipX y=shipY+20
```

### Particle System Example 1 — Celebration Confetti

```clean
canvasScene: width=800 height=600

	state:
		boolean celebrating = false

	particles "confetti":
		emitter:
			rate = 100
			burst = 50
			shape = "rect w=800 h=1"
		lifetime = 2.0..4.0
		initial:
			speed = 100..300
			angle = 60..120
			spin = -360..360
			size = 6..16
			color = ["#ff6b6b", "#feca57", "#48dbfb", "#ff9ff3", "#54a0ff", "#6bcb77"]
		overLifetime:
			size = 1.0 -> 0.6
			alpha = 1.0 -> 0.0
		forces:
			gravity = 200
			drag = 0.98
		shape = "rect"

	onPointerDown: param="x,y"
		emit particles "confetti" x=400 y=0
		celebrating = true

	draw:
		canvas.clear color="#1a1a2e"
		canvas.text value="Click to celebrate!" x=400 y=300 size=24 color="white" align="center"
```

### Particle System Example 2 — Thruster Trail

```clean
canvasScene: width=800 height=600

	state:
		number shipX = 400.0
		number shipY = 300.0

	particles "thruster":
		emitter:
			rate = 60
			shape = "circle radius=4"
		lifetime = 0.3..0.6
		initial:
			speed = 80..160
			angle = 80..100
			spin = 0..0
			size = 8..16
			color = ["#ff6b6b", "#feca57", "#ff8c00"]
		overLifetime:
			size = 1.0 -> 0.0
			alpha = 0.8 -> 0.0
		forces:
			drag = 0.92
		shape = "circle"

	onPointerMove: param="x,y"
		shipX = x
		shipY = y

	draw:
		canvas.clear color="#0f0e17"
		emit particles "thruster" x=shipX y=shipY+20
		canvas.triangle x1=shipX y1=shipY-20 x2=shipX-12 y2=shipY+16 x3=shipX+12 y3=shipY+16 color="#94a3b8"
```

---

## 17. Sprites & Animated Sprites

### Defining Animated Sprite Clips

`animSprite` clips define which frames from a spritesheet play, at what rate, and whether they loop. Define them at scene level.

```clean
animSprite "heroIdle":
	sheet = "hero"
	frames = 0..3
	fps = 8
	loop = true

animSprite "heroRun":
	sheet = "hero"
	frames = 4..11
	fps = 14
	loop = true

animSprite "heroJump":
	sheet = "hero"
	frames = 12..15
	fps = 10
	loop = false

animSprite "heroAttack":
	sheet = "hero"
	frames = 16..23
	fps = 18
	loop = false
```

### `animSprite` Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `sheet` | string | yes | Spritesheet asset name from `assets:` |
| `frames` | range `A..B` | yes | Frame indices to include (inclusive) |
| `fps` | number | yes | Playback frames per second |
| `loop` | boolean | yes | `true` = loops; `false` = plays once and freezes on last frame |

### Drawing Animated Sprites

All sprite rendering uses the unified `canvas.sprite` command (see Section 6.9). Pass `state=` to let an animState machine drive the clip, or `clip=` for a static frame.

```clean
draw:
	// Render the current frame of an animState-driven sprite
	canvas.sprite "hero" x=playerX y=playerY state="playerFSM"

	// Render with explicit clip index
	canvas.sprite "hero" x=playerX y=playerY clip=4

	// Render mirrored
	canvas.sprite "hero" x=playerX y=playerY state="playerFSM" flipX=true

	// Render scaled
	canvas.sprite "hero" x=playerX y=playerY state="playerFSM" scaleX=2.0 scaleY=2.0
```

### Complete Sprite Example

```clean
canvasScene: width=640 height=400

	assets:
		spritesheet "character" src="sprites/character.png" frameWidth=32 frameHeight=48

	state:
		number charX = 100.0
		number charY = 280.0
		boolean facingRight = true
		boolean walking = false

	animSprite "idle":
		sheet = "character"
		frames = 0..3
		fps = 6
		loop = true

	animSprite "walk":
		sheet = "character"
		frames = 4..9
		fps = 12
		loop = true

	animState "charFSM":
		state "idle":
			play animSprite "idle"
		state "walk":
			play animSprite "walk"
		transitions:
			"idle" -> "walk" when walking
			"walk" -> "idle" when walking == false
		initial = "idle"

	init:
		start animState "charFSM"

	onKeyDown: param="key"
		if key == "ArrowRight"
			walking = true
			facingRight = true
		if key == "ArrowLeft"
			walking = true
			facingRight = false

	onKeyUp: param="key"
		if key == "ArrowRight"
			walking = false
		if key == "ArrowLeft"
			walking = false

	onFrame: param="dt"
		if walking
			if facingRight
				charX = charX + 100.0 * dt
			if facingRight == false
				charX = charX - 100.0 * dt

	draw:
		canvas.clear color="#87ceeb"
		canvas.rect x=0 y=320 width=640 height=80 color="#4a7c59"
		canvas.sprite "character" x=charX y=charY state="charFSM" flipX=(facingRight == false)
```

---

## 18. Audio

Frame Canvas provides full audio control for both sound effects and music.

### Sound Effects

Sound effects use assets declared with `sound "name"` in `assets:`. Play them with `audio.play`.

```clean
// Basic playback
audio.play "jump"

// With options
audio.play "explosion" volume=0.8
audio.play "hit" pitch=1.5       // 1.0 = normal, 2.0 = double speed, 0.5 = half speed
audio.play "coin" volume=0.6 pitch=1.2
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `volume` | number | 1.0 | Playback volume: 0.0 to 1.0 |
| `pitch` | number | 1.0 | Playback rate: 1.0 = normal, >1 = faster/higher, <1 = slower/lower |

### Music

Music uses assets declared with `music "name"` in `assets:`. Music is streamed, not buffered. All music commands use the `audio.music` namespace.

```clean
// Start music
audio.music.play "theme" loop=true volume=0.5

// Stop music
audio.music.stop "theme"

// Pause and resume
audio.music.pause "theme"
audio.music.resume "theme"

// Fade out over 2 seconds
audio.music.fadeOut "theme" duration=2.0

// Crossfade between tracks
audio.music.crossfade "theme" to="boss" duration=1.5

// Set music volume live
audio.music.setVolume "theme" volume=0.3
```

| Command | Description |
|---------|-------------|
| `audio.music.play "name" loop=<bool> volume=<n>` | Start music track |
| `audio.music.stop "name"` | Stop music track |
| `audio.music.pause "name"` | Pause music track |
| `audio.music.resume "name"` | Resume paused music track |
| `audio.music.fadeOut "name" duration=<n>` | Fade out over `duration` seconds |
| `audio.music.crossfade "from" to="to" duration=<n>` | Crossfade between tracks |
| `audio.music.setVolume "name" volume=<n>` | Set track volume |

### Master Audio Controls

```clean
// Master volume (affects all sounds and music)
audio.setMasterVolume 0.8

// Mute/unmute all audio
audio.mute
audio.unmute

// Check mute state
boolean muted = audio.isMuted()
```

### Audio Example — Game with Music and SFX

```clean
canvasScene: width=800 height=600

	assets:
		sound "jump" src="sounds/jump.wav"
		sound "coin" src="sounds/coin.wav"
		sound "hurt" src="sounds/hurt.wav"
		music "level1" src="music/level1.mp3"
		music "boss" src="music/boss.mp3"

	state:
		integer score = 0
		integer health = 3
		boolean bossMode = false

	init:
		audio.music.play "level1" loop=true volume=0.5

	onKeyDown: param="key"
		if key == "Space"
			audio.play "jump" volume=0.8
		if key == "c"
			score = score + 10
			audio.play "coin" pitch=1.0 + score * 0.01
		if key == "h"
			health = health - 1
			audio.play "hurt" volume=1.0
		if key == "b"
			if bossMode == false
				bossMode = true
				audio.music.crossfade "level1" to="boss" duration=1.5
		if key == "m"
			audio.mute
		if key == "n"
			audio.unmute

	draw:
		canvas.clear color="#1a1a2e"
		canvas.text value="Score: " + score.toString() x=10 y=30 size=20 color="white"
		canvas.text value="Health: " + health.toString() x=10 y=60 size=20 color="#e94560"
```

---

## 19. Camera & Viewport

The `camera:` block defines a viewport into a larger world. The canvas is fixed in size; the camera controls what portion of the world is visible.

### Static Camera Position

```clean
camera:
	x = camX
	y = camY
	zoom = camZoom
	boundsX = 0
	boundsY = 0
	boundsWidth = 3200
	boundsHeight = 600
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `x` | number/variable | 0 | World X offset |
| `y` | number/variable | 0 | World Y offset |
| `zoom` | number/variable | 1.0 | Zoom factor |
| `boundsX` | number | 0 | World bounds minimum X |
| `boundsY` | number | 0 | World bounds minimum Y |
| `boundsWidth` | number | canvas width | World bounds total width |
| `boundsHeight` | number | canvas height | World bounds total height |

### Auto-Follow Camera

The camera can smoothly track a target position using flat camelCase attributes:

```clean
camera:
	followX = playerX
	followY = playerY
	smoothing = 0.08
	deadzoneWidth = 200
	deadzoneHeight = 100
	boundsX = 0
	boundsY = 0
	boundsWidth = 4800
	boundsHeight = 800
	zoom = 1.0
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `followX` | variable | — | World X variable to follow |
| `followY` | variable | — | World Y variable to follow |
| `smoothing` | number | 0.08 | 0.0 = instant snap, values near 1.0 = very slow |
| `offsetX` | number | 0 | Keep target offset from center (negative = left of center) |
| `offsetY` | number | 0 | Vertical follow offset |
| `deadzoneWidth` | number | 0 | Target can move this far horizontally before camera follows |
| `deadzoneHeight` | number | 0 | Target can move this far vertically before camera follows |

### Camera Shake

```clean
camera.shake intensity=8 duration=0.4 falloff="easeOut"
camera.shake intensity=20 duration=0.8 falloff="easeIn"
```

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `intensity` | number | 5 | Maximum pixel displacement |
| `duration` | number | 0.3 | Shake duration in seconds |
| `falloff` | string | `"easeOut"` | Easing name for how intensity decays |

### Camera Example — Scrolling Platformer

```clean
canvasScene: width=800 height=400 fps=60

	state:
		number playerX = 100.0
		number playerY = 280.0
		number velocityX = 0.0
		number velocityY = 0.0
		boolean grounded = true
		boolean moveRight = false
		boolean moveLeft = false

	camera:
		followX = playerX
		followY = playerY
		smoothing = 0.1
		offsetX = -200
		deadzoneWidth = 100
		deadzoneHeight = 50
		boundsX = 0
		boundsY = 0
		boundsWidth = 3200
		boundsHeight = 400

	onKeyDown: param="key"
		if key == "ArrowRight"
			moveRight = true
		if key == "ArrowLeft"
			moveLeft = true
		if key == "Space"
			if grounded
				velocityY = -500.0
				grounded = false

	onKeyUp: param="key"
		if key == "ArrowRight"
			moveRight = false
		if key == "ArrowLeft"
			moveLeft = false

	onFrame: param="dt"
		if moveRight
			velocityX = 180.0
		if moveLeft
			velocityX = -180.0
		if moveRight == false
			if moveLeft == false
				velocityX = 0.0
		velocityY = velocityY + 900.0 * dt
		playerX = playerX + velocityX * dt
		playerY = playerY + velocityY * dt
		if playerY > 300.0
			playerY = 300.0
			velocityY = 0.0
			grounded = true

	draw:
		canvas.clear color="#87ceeb"
		canvas.rect x=0 y=330 width=3200 height=70 color="#4a7c59"
		canvas.rect x=300 y=260 width=120 height=20 color="#8b4513"
		canvas.rect x=600 y=220 width=120 height=20 color="#8b4513"
		canvas.rect x=900 y=180 width=160 height=20 color="#8b4513"
		canvas.rect x=playerX-16 y=playerY-32 width=32 height=32 color="#3b82f6"
```

---

## 20. Collision Detection

Frame Canvas provides geometric collision helpers as pure functions. All return `1` (collision) or `0` (no collision), except `collision.rayCircle` which returns a distance.

### Circle vs Circle

```clean
integer hit = collision.circles x1=ballX y1=ballY r1=20 x2=enemyX y2=enemyY r2=15
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `x1`, `y1` | number | Center of first circle |
| `r1` | number | Radius of first circle |
| `x2`, `y2` | number | Center of second circle |
| `r2` | number | Radius of second circle |

### Rect vs Rect (AABB)

```clean
integer hit = collision.rects x1=playerX y1=playerY w1=32 h1=48 x2=blockX y2=blockY w2=64 h2=32
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `x1`, `y1` | number | Top-left of first rect |
| `w1`, `h1` | number | Width/height of first rect |
| `x2`, `y2` | number | Top-left of second rect |
| `w2`, `h2` | number | Width/height of second rect |

### Point in Rect

```clean
integer hit = collision.pointRect px=mouseX py=mouseY rx=btnX ry=btnY rw=120 rh=40
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `px`, `py` | number | Point position |
| `rx`, `ry` | number | Rect top-left |
| `rw`, `rh` | number | Rect dimensions |

### Circle vs Rect

```clean
integer hit = collision.circleRect cx=ballX cy=ballY r=10 rx=wallX ry=wallY rw=20 rh=200
```

### Point in Circle

```clean
integer hit = collision.pointCircle px=mouseX py=mouseY cx=circX cy=circY r=40
```

### Raycast

Returns the distance from the ray origin to the hit point, or `-1.0` if no intersection.

```clean
number dist = collision.rayCircle ox=playerX oy=playerY dx=1 dy=0 cx=enemyX cy=enemyY r=30
if dist > 0.0
	if dist < 200.0
		sighted = true
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `ox`, `oy` | number | Ray origin |
| `dx`, `dy` | number | Ray direction (normalized) |
| `cx`, `cy` | number | Circle center |
| `r` | number | Circle radius |

### Collision Example — Breakout-Style

```clean
canvasScene: width=800 height=600

	state:
		number ballX = 400.0
		number ballY = 500.0
		number ballVX = 200.0
		number ballVY = -300.0
		number ballR = 10.0
		number paddleX = 360.0
		number paddleW = 80.0
		number paddleH = 12.0
		number paddleY = 560.0
		integer score = 0

	onPointerMove: param="mx,my"
		paddleX = mx - paddleW * 0.5

	onFrame: param="dt"
		ballX = ballX + ballVX * dt
		ballY = ballY + ballVY * dt

		if ballX < ballR
			ballVX = math.abs(ballVX)
		if ballX > 800.0 - ballR
			ballVX = 0.0 - math.abs(ballVX)
		if ballY < ballR
			ballVY = math.abs(ballVY)

		integer hitPaddle = collision.circleRect cx=ballX cy=ballY r=ballR rx=paddleX ry=paddleY rw=paddleW rh=paddleH
		if hitPaddle == 1
			ballVY = 0.0 - math.abs(ballVY)
			score = score + 1

	draw:
		canvas.clear color="#1a1a2e"
		canvas.circle x=ballX y=ballY radius=ballR color="white"
		canvas.rect x=paddleX y=paddleY width=paddleW height=paddleH color="#3b82f6" cornerRadius=4
		canvas.text value="Score: " + score.toString() x=10 y=30 size=18 color="white"
```

---

## 21. Easing Functions

Frame Canvas includes a complete easing library. All easing functions accept a normalized time value `t` in range `0.0..1.0` and return an eased value in the same range (some overshoot beyond 0..1 intentionally).

### Usage in Tweens

```clean
tween "pop":
	ballRadius from=10 to=40
	duration = 0.8
	ease = "elasticOut"

tween "slide":
	panelX from=800 to=0
	duration = 0.5
	ease = "cubicInOut"
```

### Usage in Code

Easing functions can be called directly in `onFrame:` for custom interpolation:

```clean
onFrame: param="dt"
	timer = timer + dt
	number t = timer / totalDuration
	if t > 1.0
		t = 1.0
	number eased = ease.cubicInOut(t)
	ballX = startX + (endX - startX) * eased
```

### Easing Reference

#### Linear

| Function | Description |
|----------|-------------|
| `ease.linear` | Constant speed — no easing |

#### Smooth (CSS-style)

| Function | Description |
|----------|-------------|
| `ease.easeIn` | Slow start, fast end (cubic) |
| `ease.easeOut` | Fast start, slow end (cubic) |
| `ease.easeInOut` | Slow start and end, fast middle (cubic) |

#### Quad Family

| Function | Description |
|----------|-------------|
| `ease.quadIn` | Accelerate (t²) |
| `ease.quadOut` | Decelerate (t²) |
| `ease.quadInOut` | Accelerate then decelerate (t²) |

#### Cubic Family

| Function | Description |
|----------|-------------|
| `ease.cubicIn` | Accelerate (t³) |
| `ease.cubicOut` | Decelerate (t³) |
| `ease.cubicInOut` | Accelerate then decelerate (t³) |

#### Quart Family

| Function | Description |
|----------|-------------|
| `ease.quartIn` | Accelerate (t⁴) |
| `ease.quartOut` | Decelerate (t⁴) |
| `ease.quartInOut` | Accelerate then decelerate (t⁴) |

#### Quint Family

| Function | Description |
|----------|-------------|
| `ease.quintIn` | Accelerate (t⁵) |
| `ease.quintOut` | Decelerate (t⁵) |
| `ease.quintInOut` | Accelerate then decelerate (t⁵) |

#### Sine Family

| Function | Description |
|----------|-------------|
| `ease.sineIn` | Sinusoidal acceleration |
| `ease.sineOut` | Sinusoidal deceleration |
| `ease.sineInOut` | Sinusoidal in and out |

#### Expo Family

| Function | Description |
|----------|-------------|
| `ease.expoIn` | Exponential acceleration (very sharp) |
| `ease.expoOut` | Exponential deceleration |
| `ease.expoInOut` | Exponential in and out |

#### Circ Family

| Function | Description |
|----------|-------------|
| `ease.circIn` | Circular arc acceleration |
| `ease.circOut` | Circular arc deceleration |
| `ease.circInOut` | Circular arc in and out |

#### Elastic Family (spring overshoot)

| Function | Description |
|----------|-------------|
| `ease.elasticIn` | Spring windup at start |
| `ease.elasticOut` | Spring overshoot at end — best for UI pop-ins |
| `ease.elasticInOut` | Spring at both ends |

#### Bounce Family (physical bounce)

| Function | Description |
|----------|-------------|
| `ease.bounceIn` | Bounce buildup at start |
| `ease.bounceOut` | Bounce at end — simulates ball landing |
| `ease.bounceInOut` | Bounce at both ends |

#### Back Family (anticipation/overshoot)

| Function | Description |
|----------|-------------|
| `ease.backIn` | Anticipation pullback before moving forward |
| `ease.backOut` | Overshoot then settle — classic cartoon feel |
| `ease.backInOut` | Anticipation and overshoot |

### Custom Easing

Define custom cubic bezier easings with `customEase` at scene level (see Section 12).

```clean
customEase "snap" x1=0.68 y1=-0.55 x2=0.265 y2=1.55
customEase "wobbly" x1=0.36 y1=0.66 x2=-0.56 y2=1.37

tween "popIn":
	btnScale from=0 to=1
	duration = 0.5
	ease = "snap"
```

### Easing Comparison Example

```clean
canvasScene: width=700 height=420

	state:
		number timer = 0.0
		number totalTime = 2.0

	onFrame: param="dt"
		timer = timer + dt
		if timer > totalTime
			timer = 0.0

	draw:
		canvas.clear color="#1a1a2e"
		number t = timer / totalTime

		canvas.text value="linear" x=10 y=50 size=12 color="#94a3b8"
		canvas.circle x=(50.0 + ease.linear(t) * 600.0) y=50 radius=8 color="white"

		canvas.text value="cubicOut" x=10 y=90 size=12 color="#94a3b8"
		canvas.circle x=(50.0 + ease.cubicOut(t) * 600.0) y=90 radius=8 color="#3b82f6"

		canvas.text value="elasticOut" x=10 y=130 size=12 color="#94a3b8"
		canvas.circle x=(50.0 + ease.elasticOut(t) * 600.0) y=130 radius=8 color="#a78bfa"

		canvas.text value="bounceOut" x=10 y=170 size=12 color="#94a3b8"
		canvas.circle x=(50.0 + ease.bounceOut(t) * 600.0) y=170 radius=8 color="#feca57"

		canvas.text value="backOut" x=10 y=210 size=12 color="#94a3b8"
		canvas.circle x=(50.0 + ease.backOut(t) * 600.0) y=210 radius=8 color="#ff6b6b"

		canvas.text value="expoInOut" x=10 y=250 size=12 color="#94a3b8"
		canvas.circle x=(50.0 + ease.expoInOut(t) * 600.0) y=250 radius=8 color="#48dbfb"

		canvas.text value="sineInOut" x=10 y=290 size=12 color="#94a3b8"
		canvas.circle x=(50.0 + ease.sineInOut(t) * 600.0) y=290 radius=8 color="#6bcb77"
```

---

## 22. Scene Management

A canvas app can contain multiple scenes. Each scene is a separate `canvasScene:` block, typically in a separate `.cln` file in `app/canvas/scenes/`. The runtime manages scene lifecycle — only one scene is active at a time (unless using `push`/`pop`).

### Scene Transition Commands

```clean
// Replace current scene entirely (destroys old scene state)
scene.change "GameScene"

// Overlay a new scene on top (old scene keeps running in background)
scene.push "PauseMenu"

// Remove top scene, return to the one below
scene.pop

// Animated transitions
scene.changeTo "GameOver" transition="fadeOut" duration=0.5
scene.changeTo "MainMenu" transition="slideLeft" duration=0.4
scene.changeTo "NextLevel" transition="crossfade" duration=0.3
```

### Available Transitions

| Transition | Description |
|-----------|-------------|
| `"fadeOut"` | Current scene fades to black, new scene fades in |
| `"crossfade"` | Direct dissolve between scenes |
| `"slideLeft"` | New scene slides in from the right |
| `"slideRight"` | New scene slides in from the left |
| `"slideUp"` | New scene slides in from the bottom |
| `"slideDown"` | New scene slides in from the top |
| `"instant"` | No transition animation (default when not specified) |

### Passing Data Between Scenes

Use `scene.set` and `scene.get` to pass typed values when changing scenes:

```clean
// In the departing scene — set values before calling scene.change
scene.set "level" value=3
scene.set "score" value=finalScore
scene.changeTo "GameOver" transition="fadeOut" duration=0.5

// In the receiving scene — read values in init:
init:
	integer level = scene.get "level"
	integer receivedScore = scene.get "score"
```

### Sharing State with the HTML Page Layer

To share canvas state with the surrounding frame.ui page layer:

```clean
// Write a value so the page layer can read it
page.set "score" value=finalScore

// Read a value that the page layer wrote
integer currentScore = page.get "score"
```

### Scene Management Example

```clean
// app/canvas/scenes/MainMenu.cln
canvasScene: width=800 height=600 id="main-menu"

	state:
		number titleY = -100.0
		number btn1Alpha = 0.0

	tween "titleIn":
		titleY from=-100 to=200
		duration = 0.9
		ease = "elasticOut"

	tween "buttonsIn":
		btn1Alpha from=0 to=1.0
		duration = 0.5
		ease = "easeIn"
		delay = 0.7

	init:
		play tween "titleIn"
		play tween "buttonsIn"

	onPointerDown: param="x,y"
		if collision.pointRect px=x py=y rx=300 ry=320 rw=200 rh=50 == 1
			scene.changeTo "GameScene" transition="slideLeft" duration=0.4
		if collision.pointRect px=x py=y rx=300 ry=390 rw=200 rh=50 == 1
			scene.push "OptionsMenu"

	draw:
		canvas.clear color="#0f0e17"
		canvas.text value="SPACE VOYAGE" x=400 y=titleY size=52 color="white" align="center"
		canvas.alpha value=btn1Alpha:
			canvas.rect x=300 y=320 width=200 height=50 color="#3b82f6" cornerRadius=8
			canvas.text value="PLAY" x=400 y=345 size=18 color="white" align="center"
			canvas.rect x=300 y=390 width=200 height=50 color="#1e293b" cornerRadius=8
			canvas.text value="OPTIONS" x=400 y=415 size=18 color="#94a3b8" align="center"
```

---

## 23. Integration with frame.ui

Canvas files connect to HTML pages through the canvas element binding. The recommended pattern separates rendering (canvas), data (frame.data), and page layout (frame.ui) into distinct files.

### Three-File Pattern

**1. HTML page declares the canvas element:**

```html
<!-- app/pages/game.html -->
<!DOCTYPE html>
<html>
<head>
	<title>Space Voyage</title>
	<link rel="stylesheet" href="/css/style.css">
</head>
<body>
	<div class="game-container">
		<canvas id="game" width="800" height="600"></canvas>
		<div class="score-panel">
			<span>High Score: {{ highScore }}</span>
		</div>
	</div>
</body>
</html>
```

**2. Canvas file handles rendering (binds to `id="game"`):**

```clean
// app/canvas/scenes/GameScene.cln
canvasScene: width=800 height=600 id="game"

	state:
		integer score = 0

	draw:
		canvas.clear color="#0f0e17"
		canvas.text value="Score: " + score.toString() x=10 y=30 size=20 color="white"
```

**3. Backend endpoint provides data:**

```clean
// app/backend/api/game.cln
endpoints:
	get "/api/highscore":
		returns: json
		handle:
			// fetch and return high score
```

### Sharing State with frame.ui

Use `page.set` and `page.get` to project canvas state to the surrounding page layer:

```clean
onFrame: param="dt"
	score = score + 1
	page.set "gameScore" value=score    // makes score available as {{ gameScore }} in HTML
```

### Rules

- The canvas `id` attribute must match the HTML `<canvas id="...">` attribute exactly
- The canvas element dimensions in HTML must match `width` and `height` in `canvasScene:`
- CSS can style the canvas container but not the canvas itself (sizing is controlled by frame.canvas)
- frame.ui handles all DOM elements outside the canvas; frame.canvas handles everything inside it
- Do not add `<script>` tags to simulate canvas behavior — always compile the `.cln` source

---

## 24. Complete Example: Mini Space Shooter

A complete, standalone canvas game demonstrating assets, layers, animated sprites, particles, collision, audio, and all major event types.

```clean
// app/canvas/scenes/SpaceShooter.cln
canvasScene: width=800 height=600 fps=60

	assets:
		spritesheet "ship" src="sprites/ship.png" frameWidth=32 frameHeight=48
		spritesheet "enemy" src="sprites/enemy.png" frameWidth=28 frameHeight=28
		spritesheet "explosion" src="sprites/explosion.png" frameWidth=64 frameHeight=64
		sound "laser" src="sounds/laser.wav"
		sound "explode" src="sounds/explode.wav"
		sound "powerup" src="sounds/powerup.wav"
		music "battle" src="music/battle.mp3"

	layers:
		layer "background" z=0
		layer "stars" z=1
		layer "enemies" z=10
		layer "bullets" z=20
		layer "player" z=30
		layer "effects" z=40
		layer "hud" z=100

	state:
		number playerX = 400.0
		number playerY = 500.0
		number playerSpeed = 240.0
		integer playerHealth = 5
		boolean moveLeft = false
		boolean moveRight = false
		boolean moveUp = false
		boolean moveDown = false

		number shootCooldown = 0.0
		number shootRate = 0.2

		integer score = 0
		integer wave = 1
		boolean gameOver = false

		number enemyX = 400.0
		number enemyY = 80.0
		number enemyVX = 80.0
		integer enemyHealth = 3
		boolean enemyAlive = true

		number bulletX = 0.0
		number bulletY = 0.0
		boolean bulletActive = false

		number bgScrollY = 0.0
		number flashAlpha = 0.0

	animSprite "shipIdle":
		sheet = "ship"
		frames = 0..1
		fps = 6
		loop = true

	animSprite "shipThrust":
		sheet = "ship"
		frames = 2..5
		fps = 12
		loop = true

	animSprite "enemyFly":
		sheet = "enemy"
		frames = 0..3
		fps = 8
		loop = true

	animSprite "boom":
		sheet = "explosion"
		frames = 0..7
		fps = 16
		loop = false

	particles "thruster":
		emitter:
			rate = 40
			shape = "circle radius=4"
		lifetime = 0.2..0.4
		initial:
			speed = 60..120
			angle = 80..100
			size = 6..12
			color = ["#ff6b6b", "#feca57", "#ff8c00"]
		overLifetime:
			size = 1.0 -> 0.0
			alpha = 0.8 -> 0.0
		forces:
			drag = 0.90
		shape = "circle"

	particles "explosion":
		emitter:
			rate = 0
			burst = 30
			shape = "circle radius=8"
		lifetime = 0.4..0.8
		initial:
			speed = 80..300
			angle = 0..360
			spin = -180..180
			size = 4..12
			color = ["#ff6b6b", "#feca57", "#ff8c00", "white"]
		overLifetime:
			size = 1.0 -> 0.0
			alpha = 1.0 -> 0.0
		forces:
			drag = 0.94
		shape = "circle"

	init:
		audio.music.play "battle" loop=true volume=0.5

	onKeyDown: param="key"
		if key == "ArrowLeft"
			moveLeft = true
		if key == "ArrowRight"
			moveRight = true
		if key == "ArrowUp"
			moveUp = true
		if key == "ArrowDown"
			moveDown = true

	onKeyUp: param="key"
		if key == "ArrowLeft"
			moveLeft = false
		if key == "ArrowRight"
			moveRight = false
		if key == "ArrowUp"
			moveUp = false
		if key == "ArrowDown"
			moveDown = false

	onFrame: param="dt"
		if gameOver
			return

		if moveLeft
			playerX = playerX - playerSpeed * dt
		if moveRight
			playerX = playerX + playerSpeed * dt
		if moveUp
			playerY = playerY - playerSpeed * dt
		if moveDown
			playerY = playerY + playerSpeed * dt

		if playerX < 20.0
			playerX = 20.0
		if playerX > 780.0
			playerX = 780.0
		if playerY < 20.0
			playerY = 20.0
		if playerY > 580.0
			playerY = 580.0

		shootCooldown = shootCooldown - dt
		if shootCooldown < 0.0
			shootCooldown = 0.0
		if shootCooldown <= 0.0
			if bulletActive == false
				bulletX = playerX
				bulletY = playerY - 30.0
				bulletActive = true
				shootCooldown = shootRate
				audio.play "laser" volume=0.5

		if bulletActive
			bulletY = bulletY - 500.0 * dt
			if bulletY < -10.0
				bulletActive = false

		if enemyAlive
			enemyX = enemyX + enemyVX * dt
			if enemyX > 760.0
				enemyVX = 0.0 - math.abs(enemyVX)
			if enemyX < 40.0
				enemyVX = math.abs(enemyVX)

		if bulletActive
			if enemyAlive
				integer bHitE = collision.circles x1=bulletX y1=bulletY r1=6 x2=enemyX y2=enemyY r2=20
				if bHitE == 1
					bulletActive = false
					enemyHealth = enemyHealth - 1
					flashAlpha = 0.4
					if enemyHealth <= 0
						enemyAlive = false
						score = score + 100
						audio.play "explode" volume=0.8
						emit particles "explosion" x=enemyX y=enemyY

		if flashAlpha > 0.0
			flashAlpha = flashAlpha - dt * 3.0
			if flashAlpha < 0.0
				flashAlpha = 0.0

		bgScrollY = bgScrollY + 30.0 * dt
		if bgScrollY > 600.0
			bgScrollY = 0.0

		if playerHealth <= 0
			gameOver = true
			audio.music.fadeOut "battle" duration=1.5

	draw:
		on layer "background":
			canvas.clear color="#0a0a1a"
			canvas.circle x=80 y=(bgScrollY + 50.0) % 600.0 radius=1 color="white"
			canvas.circle x=200 y=(bgScrollY * 0.6 + 120.0) % 600.0 radius=1 color="#94a3b8"
			canvas.circle x=450 y=(bgScrollY + 280.0) % 600.0 radius=2 color="white"
			canvas.circle x=620 y=(bgScrollY * 0.7 + 400.0) % 600.0 radius=1 color="white"
			canvas.circle x=720 y=(bgScrollY * 0.5 + 10.0) % 600.0 radius=1 color="#94a3b8"

		on layer "enemies":
			if enemyAlive
				canvas.sprite "enemy" x=enemyX-14 y=enemyY-14 state="enemyFly"

		on layer "bullets":
			if bulletActive
				canvas.glow color="#48dbfb" blur=8:
					canvas.rect x=bulletX-3 y=bulletY-12 width=6 height=20 color="#48dbfb" cornerRadius=3

		on layer "player":
			emit particles "thruster" x=playerX y=playerY+26
			canvas.sprite "ship" x=playerX-16 y=playerY-24 state="shipThrust"

		on layer "effects":
			if flashAlpha > 0.0
				canvas.alpha value=flashAlpha:
					canvas.rect x=0 y=0 width=800 height=600 color="white"

		on layer "hud":
			canvas.text value="SCORE " + score.toString() x=10 y=30 size=18 color="white"
			canvas.text value="WAVE " + wave.toString() x=700 y=30 size=18 color="#feca57" align="right"
			canvas.circle x=10 y=580 radius=6 color=(playerHealth > 0 ? "#e94560" : "#333")
			canvas.circle x=28 y=580 radius=6 color=(playerHealth > 1 ? "#e94560" : "#333")
			canvas.circle x=46 y=580 radius=6 color=(playerHealth > 2 ? "#e94560" : "#333")
			canvas.circle x=64 y=580 radius=6 color=(playerHealth > 3 ? "#e94560" : "#333")
			canvas.circle x=82 y=580 radius=6 color=(playerHealth > 4 ? "#e94560" : "#333")

			if gameOver
				canvas.alpha value=0.7:
					canvas.rect x=0 y=0 width=800 height=600 color="black"
				canvas.text value="GAME OVER" x=400 y=260 size=52 color="#e94560" align="center"
				canvas.text value="Score: " + score.toString() x=400 y=330 size=24 color="white" align="center"
				canvas.text value="Press R to restart" x=400 y=380 size=16 color="#94a3b8" align="center"
```

---

## 25. Complete Example: Data Visualization

Demonstrating Frame Canvas for non-game uses — an animated bar chart with tweened data transitions and gradient fills.

```clean
// app/canvas/scenes/BarChart.cln
canvasScene: width=700 height=450 fps=60

	state:
		number bar1H = 0.0
		number bar2H = 0.0
		number bar3H = 0.0
		number bar4H = 0.0
		number bar5H = 0.0

		number t1 = 72.0
		number t2 = 45.0
		number t3 = 88.0
		number t4 = 31.0
		number t5 = 60.0

		string l1 = "Jan"
		string l2 = "Feb"
		string l3 = "Mar"
		string l4 = "Apr"
		string l5 = "May"

		number chartAlpha = 0.0
		number titleAlpha = 0.0

	gradient "bar1" linear x1=0 y1=0 x2=0 y2=300:
		stop 0.0 color="#a78bfa"
		stop 1.0 color="#3b82f6"

	gradient "bar2" linear x1=0 y1=0 x2=0 y2=300:
		stop 0.0 color="#34d399"
		stop 1.0 color="#0d9488"

	gradient "bar3" linear x1=0 y1=0 x2=0 y2=300:
		stop 0.0 color="#f472b6"
		stop 1.0 color="#9333ea"

	gradient "bar4" linear x1=0 y1=0 x2=0 y2=300:
		stop 0.0 color="#fbbf24"
		stop 1.0 color="#f97316"

	gradient "bar5" linear x1=0 y1=0 x2=0 y2=300:
		stop 0.0 color="#38bdf8"
		stop 1.0 color="#0ea5e9"

	timeline "chartReveal":
		at 0.0:
			animate titleAlpha to=1.0 duration=0.5 ease="easeIn"
		at 0.4:
			animate chartAlpha to=1.0 duration=0.4
		at 0.6:
			animate bar1H to=t1 duration=0.8 ease="cubicOut"
		after overlap=0.5:
			animate bar2H to=t2 duration=0.8 ease="cubicOut"
		after overlap=0.5:
			animate bar3H to=t3 duration=0.8 ease="cubicOut"
		after overlap=0.5:
			animate bar4H to=t4 duration=0.8 ease="cubicOut"
		after overlap=0.5:
			animate bar5H to=t5 duration=0.8 ease="cubicOut"

	init:
		play timeline "chartReveal"

	onPointerDown: param="x,y"
		animate bar1H to=80 duration=0.6 ease="bounceOut"
		animate bar2H to=55 duration=0.6 ease="bounceOut"
		animate bar3H to=40 duration=0.6 ease="bounceOut"
		animate bar4H to=90 duration=0.6 ease="bounceOut"
		animate bar5H to=65 duration=0.6 ease="bounceOut"

	draw:
		canvas.clear color="#0f172a"

		canvas.alpha value=titleAlpha:
			canvas.text value="Monthly Revenue" x=350 y=40 size=24 color="white" align="center"
			canvas.text value="Click bars to update" x=350 y=68 size=13 color="#64748b" align="center"

		canvas.alpha value=chartAlpha:
			canvas.line fromX=80 fromY=100 toX=660 toY=100 stroke=1 color="rgba(255,255,255,0.06)"
			canvas.line fromX=80 fromY=175 toX=660 toY=175 stroke=1 color="rgba(255,255,255,0.06)"
			canvas.line fromX=80 fromY=250 toX=660 toY=250 stroke=1 color="rgba(255,255,255,0.06)"
			canvas.line fromX=80 fromY=325 toX=660 toY=325 stroke=1 color="rgba(255,255,255,0.06)"
			canvas.line fromX=80 fromY=400 toX=660 toY=400 stroke=1 color="rgba(255,255,255,0.1)"

			canvas.text value="100%" x=70 y=103 size=11 color="#64748b" align="right"
			canvas.text value="75%" x=70 y=178 size=11 color="#64748b" align="right"
			canvas.text value="50%" x=70 y=253 size=11 color="#64748b" align="right"
			canvas.text value="25%" x=70 y=328 size=11 color="#64748b" align="right"
			canvas.text value="0%" x=70 y=403 size=11 color="#64748b" align="right"

			number barW = 80.0
			number baseY = 400.0
			number maxH = 300.0
			number x1Pos = 100.0
			number x2Pos = 216.0
			number x3Pos = 332.0
			number x4Pos = 448.0
			number x5Pos = 564.0

			number h1 = bar1H * maxH / 100.0
			canvas.rect x=x1Pos y=(baseY - h1) width=barW height=h1 gradient="bar1" cornerRadius=4

			number h2 = bar2H * maxH / 100.0
			canvas.rect x=x2Pos y=(baseY - h2) width=barW height=h2 gradient="bar2" cornerRadius=4

			number h3 = bar3H * maxH / 100.0
			canvas.rect x=x3Pos y=(baseY - h3) width=barW height=h3 gradient="bar3" cornerRadius=4

			number h4 = bar4H * maxH / 100.0
			canvas.rect x=x4Pos y=(baseY - h4) width=barW height=h4 gradient="bar4" cornerRadius=4

			number h5 = bar5H * maxH / 100.0
			canvas.rect x=x5Pos y=(baseY - h5) width=barW height=h5 gradient="bar5" cornerRadius=4

			canvas.text value=bar1H.toInteger().toString() + "%" x=(x1Pos + 40.0) y=(baseY - h1 - 10.0) size=13 color="white" align="center"
			canvas.text value=bar2H.toInteger().toString() + "%" x=(x2Pos + 40.0) y=(baseY - h2 - 10.0) size=13 color="white" align="center"
			canvas.text value=bar3H.toInteger().toString() + "%" x=(x3Pos + 40.0) y=(baseY - h3 - 10.0) size=13 color="white" align="center"
			canvas.text value=bar4H.toInteger().toString() + "%" x=(x4Pos + 40.0) y=(baseY - h4 - 10.0) size=13 color="white" align="center"
			canvas.text value=bar5H.toInteger().toString() + "%" x=(x5Pos + 40.0) y=(baseY - h5 - 10.0) size=13 color="white" align="center"

			canvas.text value=l1 x=(x1Pos + 40.0) y=418 size=13 color="#94a3b8" align="center"
			canvas.text value=l2 x=(x2Pos + 40.0) y=418 size=13 color="#94a3b8" align="center"
			canvas.text value=l3 x=(x3Pos + 40.0) y=418 size=13 color="#94a3b8" align="center"
			canvas.text value=l4 x=(x4Pos + 40.0) y=418 size=13 color="#94a3b8" align="center"
			canvas.text value=l5 x=(x5Pos + 40.0) y=418 size=13 color="#94a3b8" align="center"
```

---

## 26. Performance Guidelines

### 1. Clear Once Per Frame

Clear the canvas exactly once at the start of `draw:`. Multiple clears reset all layer buffers and waste CPU cycles. If you want a trail effect, draw a semi-transparent rectangle over the canvas instead of skipping the clear entirely.

```clean
draw:
	canvas.clear color="#0f0e17"   // once, at the top
	// ... all other drawing follows
```

### 2. Use Layers to Avoid Per-Frame Z-Sorting

Assigning draw calls to named layers eliminates the need to manually order drawing by depth each frame. The runtime composites layers in z-order once per frame — cheaper than sorting hundreds of sprites.

```clean
// Wrong: manually reordering draw calls each frame based on dynamic data
// Right: put characters on layer "characters" z=20, effects on "effects" z=30
```

### 3. Pool Particles — Don't Spawn and Destroy

Declare `particles` emitters at scene level. The runtime reuses particle slots internally. Avoid creating particle effects dynamically in loops.

```clean
// Wrong: creating new emitter definitions inside onFrame
// Right: declare once, trigger with emit / start / stop
emit particles "sparks" x=hitX y=hitY
```

### 4. Use `animSprite` Instead of Manual Frame Tracking

`animSprite` clips track elapsed time and advance frames automatically. Manually incrementing a frame counter in `onFrame:` is error-prone and frame-rate dependent.

```clean
// Wrong: spriteFrame = (spriteFrame + 1) % 8  (rate depends on fps)
// Right: animSprite "run": sheet = "hero" frames = 0..7 fps = 12 loop = true
```

### 5. Save and Restore Around Every Transform Block

Never leave transforms active between draw calls. Forgetting `canvas.restore` causes all subsequent drawing to inherit the unexpected transform, which compounds each frame.

```clean
canvas.save
canvas.translate x=enemyX y=enemyY
canvas.rotate angle=enemyAngle
// ... draw enemy
canvas.restore   // always paired
```

### 6. Pause Timelines When Offscreen

If a canvas scene is pushed under another scene (via `scene.push`), its `onFrame:` may still run. Pause expensive timelines explicitly when they are not visible.

```clean
onPause:
	pause timeline "mainLoop"

onResume:
	play timeline "mainLoop"
```

### 7. Define Gradients Once — Reference by Name

Gradients are defined at scene level, not inside `draw:`. Recreating gradient objects every frame is expensive. Name them once in the scene body and reference with `gradient=` in draw.

```clean
// Wrong: creating gradient logic inside draw: block
// Right: gradient "glow" radial cx=400 cy=300 r=200: ... (at scene level)
canvas.circle x=400 y=300 radius=200 gradient="glow"
```

### 8. Minimize Shadow and Blur Blocks

`canvas.shadow` and `canvas.blur` trigger compositing passes that are expensive on mobile and lower-end devices. Wrap only the specific elements that need the effect, not the entire draw block.

### 9. Use `collision.*` Functions Sparingly in `onFrame:`

Collision checks run every frame. For scenes with many objects, batch collision checks and early-exit when objects are clearly distant (broad-phase check with `collision.circles` using a large outer radius before checking precise geometry).

### 10. Keep Draw Calls Under ~500 Per Frame

Each `canvas.*` call crosses the WASM-to-host bridge. At 60fps, exceeding 500 calls per frame increases per-frame overhead noticeably. Group draw calls using paths when drawing many similar shapes.

---

## 27. Bridge Function Reference

All `canvas.*`, `audio.*`, `collision.*`, `scene.*`, `ease.*`, and `animSprite.*` commands compile to bridge function calls. The following tables are the complete bridge contract.

### Lifecycle

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_init` | `width: i32, height: i32, id: ptr, idLen: i32` | `i32` | Initialize canvas surface, returns canvas ID |
| `_canvas_clear` | `canvasId: i32` | `i32` | Clear with current background color |
| `_canvas_clear_color` | `canvasId: i32, color: ptr, colorLen: i32` | `i32` | Clear with specified color |
| `_canvas_present` | `canvasId: i32` | `i32` | Flush frame to screen |
| `_canvas_set_fps` | `canvasId: i32, fps: i32` | `i32` | Set target frame rate |
| `_canvas_request_frame` | `canvasId: i32` | `i32` | Start animation loop |
| `_canvas_cancel_frame` | `frameId: i32` | `i32` | Stop animation loop |
| `_canvas_get_delta_time` | none | `f64` | Delta time in seconds since last frame |
| `_canvas_get_time` | none | `f64` | Elapsed time in seconds since scene start |

### Shapes

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_circle_filled` | `id, x, y, r, color:ptr+len` | `i32` | Filled circle |
| `_canvas_circle_outline` | `id, x, y, r, color:ptr+len` | `i32` | Circle outline |
| `_canvas_rect_filled` | `id, x, y, w, h, color:ptr+len` | `i32` | Filled rectangle |
| `_canvas_rect_outline` | `id, x, y, w, h, color:ptr+len` | `i32` | Rectangle outline |
| `_canvas_rect_rounded` | `id, x, y, w, h, r, color:ptr+len` | `i32` | Rounded rectangle |
| `_canvas_rect_gradient` | `id, x, y, w, h, r, gradientId: i32` | `i32` | Rounded rectangle filled with named gradient |
| `_canvas_circle_gradient` | `id, x, y, r, gradientId: i32` | `i32` | Circle filled with named gradient |
| `_canvas_line` | `id, x1, y1, x2, y2, stroke, color:ptr+len` | `i32` | Straight line |
| `_canvas_triangle_filled` | `id, x1, y1, x2, y2, x3, y3, color:ptr+len` | `i32` | Filled triangle |
| `_canvas_triangle_outline` | `id, x1, y1, x2, y2, x3, y3, color:ptr+len` | `i32` | Triangle outline |
| `_canvas_ellipse_filled` | `id, x, y, rx, ry, color:ptr+len` | `i32` | Filled ellipse |
| `_canvas_ellipse_outline` | `id, x, y, rx, ry, color:ptr+len` | `i32` | Ellipse outline |

### Paths

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_begin_path` | `canvasId: i32` | `i32` | Start new path |
| `_canvas_move_to` | `id, x, y` | `i32` | Move pen |
| `_canvas_line_to` | `id, x, y` | `i32` | Line segment |
| `_canvas_curve_to` | `id, cpx, cpy, x, y` | `i32` | Quadratic bezier |
| `_canvas_cubic_to` | `id, cp1x, cp1y, cp2x, cp2y, x, y` | `i32` | Cubic bezier |
| `_canvas_arc` | `id, x, y, r, startAngle, endAngle` | `i32` | Arc segment |
| `_canvas_close_path` | `canvasId: i32` | `i32` | Close path |
| `_canvas_fill_path` | `id, color:ptr+len` | `i32` | Fill closed path |
| `_canvas_stroke_path` | `id, color:ptr+len, width` | `i32` | Stroke path |
| `_canvas_draw_named_path` | `id, name:ptr+len, stroke, color:ptr+len` | `i32` | Draw a named `path` |

### Text

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_text` | `id, text:ptr+len, x, y, size, color:ptr+len` | `i32` | Draw text |
| `_canvas_text_aligned` | `id, text:ptr+len, x, y, size, color:ptr+len, align:ptr+len, baseline:ptr+len` | `i32` | Draw text with alignment |
| `_canvas_text_font` | `id, text:ptr+len, x, y, size, color:ptr+len, font:ptr+len, align:ptr+len, baseline:ptr+len` | `i32` | Draw text with font asset |
| `_canvas_text_width` | `id, text:ptr+len, size` | `f64` | Measure text width |

### Images & Sprites

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_image` | `id, src:ptr+len, x, y, w, h` | `i32` | Draw image asset |
| `_canvas_image_opacity` | `id, src:ptr+len, x, y, w, h, opacity` | `i32` | Draw image with opacity |
| `_sprite_draw` | `id, sheet:ptr+len, clip, x, y, scaleX, scaleY, flipX, flipY, rotation, alpha` | `i32` | Draw sprite frame with full options |
| `_sprite_draw_tint` | `id, sheet:ptr+len, clip, x, y, scaleX, scaleY, rotation, alpha, tint:ptr+len, tintStrength` | `i32` | Draw sprite with tint |
| `_sprite_draw_state` | `id, sheet:ptr+len, fsmName:ptr+len, x, y, scaleX, scaleY, flipX, flipY, rotation, alpha` | `i32` | Draw animState-driven sprite |
| `_anim_sprite_reset` | `name:ptr+len` | `i32` | Reset animSprite to first frame |

### Transforms & Groups

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_save` | `canvasId: i32` | `i32` | Push transform stack |
| `_canvas_restore` | `canvasId: i32` | `i32` | Pop transform stack |
| `_canvas_translate` | `id, x, y` | `i32` | Translate origin |
| `_canvas_rotate` | `id, angleDegrees` | `i32` | Rotate (degrees, clockwise) |
| `_canvas_scale` | `id, sx, sy` | `i32` | Scale |
| `_canvas_group_begin` | `id, x, y, rotation, scaleX, scaleY, alpha` | `i32` | Begin transform group |
| `_canvas_group_end` | `canvasId: i32` | `i32` | End transform group |

### Clip / Mask

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_clip_rect_begin` | `id, x, y, w, h` | `i32` | Begin rectangular clip region |
| `_canvas_clip_circle_begin` | `id, x, y, radius` | `i32` | Begin circular clip region |
| `_canvas_clip_path_begin` | `id, name:ptr+len` | `i32` | Begin path-based clip region |
| `_canvas_clip_end` | `canvasId: i32` | `i32` | End clip region |

### Effects

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_shadow_begin` | `id, blur, offsetX, offsetY, color:ptr+len` | `i32` | Begin shadow scope |
| `_canvas_shadow_end` | `canvasId: i32` | `i32` | End shadow scope |
| `_canvas_blur_begin` | `id, radius` | `i32` | Begin blur scope |
| `_canvas_blur_end` | `canvasId: i32` | `i32` | End blur scope |
| `_canvas_alpha_begin` | `id, alpha` | `i32` | Begin alpha scope |
| `_canvas_alpha_end` | `canvasId: i32` | `i32` | End alpha scope |
| `_canvas_blend_begin` | `id, mode:ptr+len` | `i32` | Begin blend mode scope |
| `_canvas_blend_end` | `canvasId: i32` | `i32` | End blend mode scope |
| `_canvas_glow_begin` | `id, color:ptr+len, blur` | `i32` | Begin glow scope |
| `_canvas_glow_end` | `canvasId: i32` | `i32` | End glow scope |

### Gradients

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_gradient_linear` | `name:ptr+len, x1, y1, x2, y2` | `i32` | Create linear gradient |
| `_gradient_radial` | `name:ptr+len, cx, cy, r` | `i32` | Create radial gradient |
| `_gradient_add_stop` | `name:ptr+len, position, color:ptr+len` | `i32` | Add color stop |
| `_gradient_ref` | `name:ptr+len` | `i32` | Get gradient reference ID for fill |

### Layers

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_layer_declare` | `name:ptr+len, z` | `i32` | Declare a named layer |
| `_layer_begin` | `name:ptr+len` | `i32` | Begin drawing to layer |
| `_layer_end` | none | `i32` | End drawing to layer |

### Animation (Tweens & Timelines)

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_tween_play` | `name:ptr+len` | `i32` | Play a named tween |
| `_tween_stop` | `name:ptr+len` | `i32` | Stop a named tween |
| `_tween_pause` | `name:ptr+len` | `i32` | Pause a named tween |
| `_tween_resume` | `name:ptr+len` | `i32` | Resume a paused tween |
| `_tween_animate` | `varName:ptr+len, from, to, duration, easingId, delay` | `i32` | Inline animate |
| `_tween_animate_path` | `xVar:ptr+len, yVar:ptr+len, pathName:ptr+len, duration, easingId, repeat, orient` | `i32` | Path-following animate |
| `_timeline_play` | `name:ptr+len` | `i32` | Play a named timeline |
| `_timeline_stop` | `name:ptr+len` | `i32` | Stop a timeline |
| `_timeline_pause` | `name:ptr+len` | `i32` | Pause a timeline |
| `_timeline_resume` | `name:ptr+len` | `i32` | Resume a paused timeline |
| `_timeline_seek` | `name:ptr+len, position` | `i32` | Seek timeline to position (seconds) |
| `_anim_state_start` | `name:ptr+len` | `i32` | Start an animation state machine |
| `_anim_state_force` | `name:ptr+len, state:ptr+len` | `i32` | Force FSM into named state |
| `_anim_state_current` | `name:ptr+len` | `ptr` | Get current FSM state name |
| `_custom_ease_register` | `name:ptr+len, x1, y1, x2, y2` | `i32` | Register custom cubic bezier easing |

### Easing

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_ease_linear` | `t: f64` | `f64` | Linear easing |
| `_ease_cubic_in` | `t: f64` | `f64` | Cubic ease in |
| `_ease_cubic_out` | `t: f64` | `f64` | Cubic ease out |
| `_ease_cubic_in_out` | `t: f64` | `f64` | Cubic ease in-out |
| `_ease_elastic_out` | `t: f64` | `f64` | Elastic ease out |
| `_ease_bounce_out` | `t: f64` | `f64` | Bounce ease out |
| `_ease_back_out` | `t: f64` | `f64` | Back ease out |
| `_ease_custom` | `name:ptr+len, t: f64` | `f64` | Custom named cubic bezier easing |
| `_ease_*` | `t: f64` | `f64` | All easing families follow same signature |

### Audio — Sound Effects

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_audio_play_sound` | `name:ptr+len, volume, pitch` | `i32` | Play sound effect |
| `_audio_preload` | `name:ptr+len, src:ptr+len` | `i32` | Preload audio asset |

### Audio — Music

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_audio_music_play` | `name:ptr+len, loop, volume` | `i32` | Start music track |
| `_audio_music_stop` | `name:ptr+len` | `i32` | Stop music track |
| `_audio_music_pause` | `name:ptr+len` | `i32` | Pause music track |
| `_audio_music_resume` | `name:ptr+len` | `i32` | Resume music track |
| `_audio_music_fade_out` | `name:ptr+len, duration` | `i32` | Fade out music |
| `_audio_music_crossfade` | `from:ptr+len, to:ptr+len, duration` | `i32` | Crossfade tracks |
| `_audio_music_set_volume` | `name:ptr+len, volume` | `i32` | Set track volume |
| `_audio_set_master_volume` | `volume` | `i32` | Set master volume |
| `_audio_mute` | none | `i32` | Mute all audio |
| `_audio_unmute` | none | `i32` | Unmute all audio |
| `_audio_is_muted` | none | `i32` | Returns 1 if muted |

### Collision Detection

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_collision_circles` | `x1, y1, r1, x2, y2, r2` | `i32` | Circle vs circle (1=hit) |
| `_collision_rects` | `x1, y1, w1, h1, x2, y2, w2, h2` | `i32` | AABB vs AABB (1=hit) |
| `_collision_point_rect` | `px, py, rx, ry, rw, rh` | `i32` | Point in rect (1=hit) |
| `_collision_circle_rect` | `cx, cy, r, rx, ry, rw, rh` | `i32` | Circle vs rect (1=hit) |
| `_collision_point_circle` | `px, py, cx, cy, r` | `i32` | Point in circle (1=hit) |
| `_collision_ray_circle` | `ox, oy, dx, dy, cx, cy, r` | `f64` | Ray vs circle distance (-1=miss) |

### Camera

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_camera_set_position` | `x, y` | `i32` | Set camera world position |
| `_camera_set_zoom` | `zoom` | `i32` | Set zoom factor |
| `_camera_set_follow` | `followX, followY, smoothing` | `i32` | Enable follow with smoothing |
| `_camera_set_offset` | `offsetX, offsetY` | `i32` | Follow offset |
| `_camera_set_deadzone` | `width, height` | `i32` | Follow deadzone dimensions |
| `_camera_set_bounds` | `x, y, width, height` | `i32` | World boundary limits |
| `_camera_shake` | `intensity, duration, falloffId` | `i32` | Trigger camera shake |

### Particles

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_particles_emit` | `name:ptr+len, x, y` | `i32` | One-shot burst emit |
| `_particles_start` | `name:ptr+len, x, y` | `i32` | Start continuous emitter |
| `_particles_stop` | `name:ptr+len` | `i32` | Stop emitter |

### Input Events

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_canvas_event_x` | none | `f64` | Pointer X (inside event handler) |
| `_canvas_event_y` | none | `f64` | Pointer Y (inside event handler) |
| `_canvas_event_key` | none | `ptr` | Key string (inside key handler) |
| `_input_key_down` | `key:ptr+len` | `i32` | Poll: is key currently held (1=yes) |

### Scene Management

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_scene_change` | `name:ptr+len` | `i32` | Replace current scene |
| `_scene_change_animated` | `name:ptr+len, transition:ptr+len, duration` | `i32` | Animated scene change |
| `_scene_push` | `name:ptr+len` | `i32` | Push scene onto stack |
| `_scene_pop` | none | `i32` | Pop top scene |
| `_scene_set_int` | `key:ptr+len, value: i32` | `i32` | Write integer value for receiving scene |
| `_scene_set_float` | `key:ptr+len, value: f64` | `i32` | Write float value for receiving scene |
| `_scene_set_string` | `key:ptr+len, value:ptr+len` | `i32` | Write string value for receiving scene |
| `_scene_get_int` | `key:ptr+len` | `i32` | Read integer value passed from previous scene |
| `_scene_get_float` | `key:ptr+len` | `f64` | Read float value passed from previous scene |
| `_scene_get_string` | `key:ptr+len` | `ptr` | Read string value passed from previous scene |
| `_page_set_int` | `key:ptr+len, value: i32` | `i32` | Write integer value to page layer |
| `_page_set_float` | `key:ptr+len, value: f64` | `i32` | Write float value to page layer |
| `_page_get_int` | `key:ptr+len` | `i32` | Read integer value from page layer |
| `_page_get_float` | `key:ptr+len` | `f64` | Read float value from page layer |

### Font Assets

| Bridge Function | Parameters | Returns | Description |
|----------------|-----------|---------|-------------|
| `_font_load` | `name:ptr+len, src:ptr+len` | `i32` | Load font asset from URL |

---

## 28. Version History

| Version | Changes |
|---------|---------|
| 2.1.0 | Syntax correction release: `=` for all leaf values (no `:` for values); unified `canvas.sprite` replaces `animSprite.draw` and `canvas.animSprite`; tween variable-first format; `when` keyword in animState transitions; `onComplete "state"` without arrow; `after overlap=`/`after gap=` replace negative `after`; `emitter:` replaces `emit:` sub-block; `overLifetime` camelCase replaces `over_lifetime`; particle render attributes flattened; `animState.force` uses `to=`; `gradient=` attribute replaces `fill=gradient()`; `audio.play` for SFX, `audio.music.*` for music; `collision.circles`/`collision.rects`/`collision.rayCircle` clean names; camera flattened to camelCase attributes; path commands without colon; `customEase` flat declaration; added `canvas.group`, `canvas.clip`, `onExit`/`onPause`/`onResume` lifecycle hooks, `scene.set`/`scene.get`, `page.set`/`page.get`, font assets, `together stagger=`, particle `forces:` sub-block |
| 2.0.0 | Complete rewrite: tween system, named timelines, animation state machines, particle systems, path animation, named layers, camera/viewport, gradients, effect blocks (shadow, blur, glow, alpha, blendMode), full easing library, scene management with transitions, expanded audio API (crossfade, fadeOut, pitch), collision detection suite, animSprite system, complete bridge function reference |
| 1.1.0 | Added bridge function reference, plugin manifest details |
| 1.0.0 | Initial release — shapes, transforms, basic animation, input handlers |

---

**End of Frame Canvas Specification (12)**
