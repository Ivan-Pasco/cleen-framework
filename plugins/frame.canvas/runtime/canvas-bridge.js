/**
 * Clean Canvas Bridge for WebAssembly
 * Version: 2.0.0
 *
 * Provides 130+ bridge functions for game development:
 * - Canvas lifecycle and rendering
 * - Shapes, text, images
 * - Transforms
 * - Animation loop
 * - Audio (sound effects and music)
 * - Sprites and animation
 * - Input (mouse, keyboard, touch, gamepad)
 * - Collision detection
 * - Asset management
 * - Camera system
 * - Gradients and paths
 * - Advanced rendering (blend modes, shadows)
 * - Easing functions
 * - Scene management
 */

class CanvasBridge {
  constructor() {
    // ========================================================================
    // CANVAS STATE
    // ========================================================================
    this.canvases = new Map();
    this.nextCanvasId = 1;

    // ========================================================================
    // ANIMATION STATE
    // ========================================================================
    this.animationFrames = new Map();
    this.nextFrameId = 1;
    this.lastFrameTime = performance.now();
    this.startTime = performance.now();
    this.deltaTime = 0;
    this.fps = 60;
    this.frameCount = 0;
    this.fpsLastTime = performance.now();

    // ========================================================================
    // AUDIO STATE
    // ========================================================================
    this.audioContext = null;
    this.sounds = new Map();
    this.nextSoundId = 1;
    this.soundInstances = new Map();
    this.nextInstanceId = 1;
    this.music = new Map();
    this.nextMusicId = 1;
    this.currentMusic = null;
    this.currentMusicElement = null;
    this.masterVolume = 1.0;
    this.musicVolume = 1.0;
    this.isMuted = false;

    // ========================================================================
    // SPRITE STATE
    // ========================================================================
    this.spriteSheets = new Map();
    this.nextSpriteId = 1;

    // ========================================================================
    // INPUT STATE
    // ========================================================================
    // Mouse
    this.mouseX = 0;
    this.mouseY = 0;
    this.mouseButtons = [false, false, false];
    this.mouseButtonsJustPressed = [false, false, false];
    this.mouseButtonsJustReleased = [false, false, false];
    this.mouseWheelX = 0;
    this.mouseWheelY = 0;

    // Keyboard
    this.keysDown = new Set();
    this.keysJustPressed = new Set();
    this.keysJustReleased = new Set();
    this.lastKey = '';
    this.textInput = '';

    // Touch
    this.touches = [];
    this.touchStarted = false;
    this.touchEnded = false;

    // Gamepad
    this.gamepads = new Map();
    this.gamepadButtonsJustPressed = new Map();

    // ========================================================================
    // ASSET STATE
    // ========================================================================
    this.assets = new Map();
    this.assetQueue = [];
    this.assetsLoaded = 0;
    this.assetsTotal = 0;
    this.imageCache = new Map();

    // ========================================================================
    // CAMERA STATE
    // ========================================================================
    this.cameraX = 0;
    this.cameraY = 0;
    this.cameraZoom = 1.0;
    this.cameraRotation = 0;
    this.cameraShakeIntensity = 0;
    this.cameraShakeDuration = 0;
    this.cameraShakeStart = 0;

    // ========================================================================
    // GRADIENT STATE
    // ========================================================================
    this.gradients = new Map();
    this.nextGradientId = 1;

    // ========================================================================
    // SCENE STATE
    // ========================================================================
    this.currentScene = 'default';
    this.sceneStack = [];

    // ========================================================================
    // WASM REFERENCES
    // ========================================================================
    this.memory = null;
    this.wasmInstance = null;

    // Initialize audio context on first user interaction
    this._initAudioOnInteraction();
  }

  // ==========================================================================
  // INITIALIZATION
  // ==========================================================================

  _initAudioOnInteraction() {
    const initAudio = () => {
      if (!this.audioContext) {
        this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
        console.log('[CanvasBridge] Audio context initialized');
      }
      document.removeEventListener('click', initAudio);
      document.removeEventListener('keydown', initAudio);
      document.removeEventListener('touchstart', initAudio);
    };
    document.addEventListener('click', initAudio);
    document.addEventListener('keydown', initAudio);
    document.addEventListener('touchstart', initAudio);
  }

  // ==========================================================================
  // MEMORY HELPERS
  // ==========================================================================

  readString(ptr, len) {
    if (!this.memory) return '';
    const bytes = new Uint8Array(this.memory.buffer, ptr, len);
    return new TextDecoder().decode(bytes);
  }

  writeString(str, ptr) {
    if (!this.memory) return 0;
    const bytes = new TextEncoder().encode(str);
    const view = new Uint8Array(this.memory.buffer, ptr, bytes.length);
    view.set(bytes);
    return bytes.length;
  }

  // ==========================================================================
  // CANVAS HELPERS
  // ==========================================================================

  getCanvas(id) {
    const entry = this.canvases.get(id);
    return entry ? entry.ctx : null;
  }

  getCanvasElement(id) {
    const entry = this.canvases.get(id);
    return entry ? entry.element : null;
  }

  // ==========================================================================
  // INPUT SETUP
  // ==========================================================================

  setupInputHandlers(canvasId, element) {
    const self = this;

    // Make canvas focusable
    element.setAttribute('tabindex', '0');
    element.style.outline = 'none';

    // ========== MOUSE EVENTS ==========
    element.addEventListener('mousemove', (e) => {
      const rect = element.getBoundingClientRect();
      self.mouseX = e.clientX - rect.left;
      self.mouseY = e.clientY - rect.top;
    });

    element.addEventListener('mousedown', (e) => {
      const button = e.button;
      if (button < 3) {
        self.mouseButtons[button] = true;
        self.mouseButtonsJustPressed[button] = true;
      }
      element.focus();
    });

    element.addEventListener('mouseup', (e) => {
      const button = e.button;
      if (button < 3) {
        self.mouseButtons[button] = false;
        self.mouseButtonsJustReleased[button] = true;
      }
    });

    element.addEventListener('mouseleave', () => {
      self.mouseButtons = [false, false, false];
    });

    element.addEventListener('wheel', (e) => {
      self.mouseWheelX = e.deltaX;
      self.mouseWheelY = e.deltaY;
    });

    element.addEventListener('contextmenu', (e) => e.preventDefault());

    // ========== KEYBOARD EVENTS ==========
    element.addEventListener('keydown', (e) => {
      const key = e.key;
      if (!self.keysDown.has(key)) {
        self.keysJustPressed.add(key);
      }
      self.keysDown.add(key);
      self.lastKey = key;

      // Capture text input for printable characters
      if (key.length === 1) {
        self.textInput += key;
      }

      // Prevent default for game keys
      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', ' '].includes(key)) {
        e.preventDefault();
      }
    });

    element.addEventListener('keyup', (e) => {
      const key = e.key;
      self.keysDown.delete(key);
      self.keysJustReleased.add(key);
    });

    // ========== TOUCH EVENTS ==========
    element.addEventListener('touchstart', (e) => {
      e.preventDefault();
      self.touchStarted = true;
      self._updateTouches(e.touches, element);
      // Simulate mouse for primary touch
      if (e.touches.length > 0) {
        const rect = element.getBoundingClientRect();
        self.mouseX = e.touches[0].clientX - rect.left;
        self.mouseY = e.touches[0].clientY - rect.top;
        self.mouseButtons[0] = true;
        self.mouseButtonsJustPressed[0] = true;
      }
    });

    element.addEventListener('touchmove', (e) => {
      e.preventDefault();
      self._updateTouches(e.touches, element);
      if (e.touches.length > 0) {
        const rect = element.getBoundingClientRect();
        self.mouseX = e.touches[0].clientX - rect.left;
        self.mouseY = e.touches[0].clientY - rect.top;
      }
    });

    element.addEventListener('touchend', (e) => {
      e.preventDefault();
      self.touchEnded = true;
      self._updateTouches(e.touches, element);
      if (e.touches.length === 0) {
        self.mouseButtons[0] = false;
        self.mouseButtonsJustReleased[0] = true;
      }
    });

    element.addEventListener('touchcancel', (e) => {
      self.touchEnded = true;
      self._updateTouches(e.touches, element);
      self.mouseButtons[0] = false;
    });

    // ========== GAMEPAD POLLING ==========
    // Gamepads are polled each frame, not event-driven
  }

  _updateTouches(touchList, element) {
    const rect = element.getBoundingClientRect();
    this.touches = [];
    for (let i = 0; i < touchList.length; i++) {
      const touch = touchList[i];
      this.touches.push({
        id: touch.identifier,
        x: touch.clientX - rect.left,
        y: touch.clientY - rect.top
      });
    }
  }

  _pollGamepads() {
    const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
    for (let i = 0; i < gamepads.length; i++) {
      const gp = gamepads[i];
      if (gp) {
        const prev = this.gamepads.get(i) || { buttons: [] };
        const justPressed = [];

        for (let b = 0; b < gp.buttons.length; b++) {
          const wasPressed = prev.buttons[b] || false;
          const isPressed = gp.buttons[b].pressed;
          justPressed[b] = isPressed && !wasPressed;
        }

        this.gamepads.set(i, {
          connected: true,
          buttons: gp.buttons.map(b => b.pressed),
          axes: [...gp.axes]
        });
        this.gamepadButtonsJustPressed.set(i, justPressed);
      }
    }
  }

  _clearFrameInputState() {
    // Clear "just pressed/released" states at end of frame
    this.mouseButtonsJustPressed = [false, false, false];
    this.mouseButtonsJustReleased = [false, false, false];
    this.mouseWheelX = 0;
    this.mouseWheelY = 0;
    this.keysJustPressed.clear();
    this.keysJustReleased.clear();
    this.textInput = '';
    this.touchStarted = false;
    this.touchEnded = false;
  }

  // ==========================================================================
  // GET IMPORTS - Returns all bridge functions
  // ==========================================================================

  getImports(memory) {
    this.memory = memory;
    const self = this;

    return {
      env: {
        // ====================================================================
        // CANVAS LIFECYCLE
        // ====================================================================

        _canvas_init(width, height) {
          const id = self.nextCanvasId++;
          const canvas = document.createElement('canvas');
          canvas.width = width;
          canvas.height = height;
          canvas.id = `clean-canvas-${id}`;
          canvas.style.display = 'block';

          const ctx = canvas.getContext('2d');

          self.canvases.set(id, {
            element: canvas,
            ctx: ctx,
            width: width,
            height: height
          });

          self.setupInputHandlers(id, canvas);

          const container = document.getElementById('canvas-container') || document.body;
          container.appendChild(canvas);

          console.log(`[CanvasBridge] Canvas ${id} created: ${width}x${height}`);
          return id;
        },

        _canvas_clear(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.clearRect(0, 0, entry.width, entry.height);
          return 0;
        },

        _canvas_clear_color(canvasId, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          entry.ctx.fillStyle = color;
          entry.ctx.fillRect(0, 0, entry.width, entry.height);
          return 0;
        },

        _canvas_present(canvasId) {
          return self.canvases.has(canvasId) ? 0 : -1;
        },

        _canvas_resize(canvasId, width, height) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.element.width = width;
          entry.element.height = height;
          entry.width = width;
          entry.height = height;
          return 0;
        },

        _canvas_get_width(canvasId) {
          const entry = self.canvases.get(canvasId);
          return entry ? entry.width : 0;
        },

        _canvas_get_height(canvasId) {
          const entry = self.canvases.get(canvasId);
          return entry ? entry.height : 0;
        },

        // ====================================================================
        // BASIC SHAPES
        // ====================================================================

        _canvas_circle(canvasId, x, y, radius, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.arc(x, y, radius, 0, Math.PI * 2);
          ctx.strokeStyle = color;
          ctx.stroke();
          return 0;
        },

        _canvas_circle_filled(canvasId, x, y, radius, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.arc(x, y, radius, 0, Math.PI * 2);
          ctx.fillStyle = color;
          ctx.fill();
          return 0;
        },

        _canvas_rect(canvasId, x, y, width, height, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          entry.ctx.strokeStyle = color;
          entry.ctx.strokeRect(x, y, width, height);
          return 0;
        },

        _canvas_rect_filled(canvasId, x, y, width, height, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          entry.ctx.fillStyle = color;
          entry.ctx.fillRect(x, y, width, height);
          return 0;
        },

        _canvas_rect_rounded(canvasId, x, y, width, height, radius, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.roundRect(x, y, width, height, radius);
          ctx.strokeStyle = color;
          ctx.stroke();
          return 0;
        },

        _canvas_rect_rounded_filled(canvasId, x, y, width, height, radius, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.roundRect(x, y, width, height, radius);
          ctx.fillStyle = color;
          ctx.fill();
          return 0;
        },

        _canvas_line(canvasId, x1, y1, x2, y2, strokeWidth, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.lineTo(x2, y2);
          ctx.strokeStyle = color;
          ctx.lineWidth = strokeWidth;
          ctx.stroke();
          return 0;
        },

        _canvas_ellipse(canvasId, x, y, radiusX, radiusY, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.ellipse(x, y, radiusX, radiusY, 0, 0, Math.PI * 2);
          ctx.strokeStyle = color;
          ctx.stroke();
          return 0;
        },

        _canvas_ellipse_filled(canvasId, x, y, radiusX, radiusY, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.ellipse(x, y, radiusX, radiusY, 0, 0, Math.PI * 2);
          ctx.fillStyle = color;
          ctx.fill();
          return 0;
        },

        _canvas_triangle(canvasId, x1, y1, x2, y2, x3, y3, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.lineTo(x2, y2);
          ctx.lineTo(x3, y3);
          ctx.closePath();
          ctx.strokeStyle = color;
          ctx.stroke();
          return 0;
        },

        _canvas_triangle_filled(canvasId, x1, y1, x2, y2, x3, y3, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.lineTo(x2, y2);
          ctx.lineTo(x3, y3);
          ctx.closePath();
          ctx.fillStyle = color;
          ctx.fill();
          return 0;
        },

        _canvas_polygon(canvasId, pointsPtr, colorPtr, colorLen) {
          // Points are stored as pairs of f64 in WASM memory
          // Format: [x1, y1, x2, y2, ..., 0, 0] (terminated by 0,0)
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;

          const view = new Float64Array(self.memory.buffer, pointsPtr);
          ctx.beginPath();
          let i = 0;
          let first = true;
          while (i < view.length - 1) {
            const x = view[i];
            const y = view[i + 1];
            if (x === 0 && y === 0 && !first) break;
            if (first) {
              ctx.moveTo(x, y);
              first = false;
            } else {
              ctx.lineTo(x, y);
            }
            i += 2;
          }
          ctx.closePath();
          ctx.strokeStyle = color;
          ctx.stroke();
          return 0;
        },

        _canvas_polygon_filled(canvasId, pointsPtr, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;

          const view = new Float64Array(self.memory.buffer, pointsPtr);
          ctx.beginPath();
          let i = 0;
          let first = true;
          while (i < view.length - 1) {
            const x = view[i];
            const y = view[i + 1];
            if (x === 0 && y === 0 && !first) break;
            if (first) {
              ctx.moveTo(x, y);
              first = false;
            } else {
              ctx.lineTo(x, y);
            }
            i += 2;
          }
          ctx.closePath();
          ctx.fillStyle = color;
          ctx.fill();
          return 0;
        },

        // ====================================================================
        // TEXT RENDERING
        // ====================================================================

        _canvas_text(canvasId, textPtr, textLen, x, y, size, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const text = self.readString(textPtr, textLen);
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.font = `${size}px sans-serif`;
          ctx.fillStyle = color;
          ctx.fillText(text, x, y);
          return 0;
        },

        _canvas_text_number(canvasId, number, x, y, size, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          const ctx = entry.ctx;
          ctx.font = `${size}px sans-serif`;
          ctx.fillStyle = color;
          ctx.fillText(number.toString(), x, y);
          return 0;
        },

        _canvas_text_font(canvasId, textPtr, textLen, x, y, size, colorPtr, colorLen, fontPtr, fontLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const text = self.readString(textPtr, textLen);
          const color = self.readString(colorPtr, colorLen);
          const font = self.readString(fontPtr, fontLen);
          const ctx = entry.ctx;
          ctx.font = `${size}px ${font}`;
          ctx.fillStyle = color;
          ctx.fillText(text, x, y);
          return 0;
        },

        _canvas_text_align(canvasId, alignPtr, alignLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const align = self.readString(alignPtr, alignLen);
          entry.ctx.textAlign = align;
          return 0;
        },

        _canvas_text_baseline(canvasId, baselinePtr, baselineLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const baseline = self.readString(baselinePtr, baselineLen);
          entry.ctx.textBaseline = baseline;
          return 0;
        },

        _canvas_measure_text(canvasId, textPtr, textLen, fontSize) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return 0;
          const text = self.readString(textPtr, textLen);
          const ctx = entry.ctx;
          const prevFont = ctx.font;
          ctx.font = `${fontSize}px sans-serif`;
          const width = ctx.measureText(text).width;
          ctx.font = prevFont;
          return width;
        },

        // ====================================================================
        // IMAGES
        // ====================================================================

        _canvas_image(canvasId, srcPtr, srcLen, x, y, width, height) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const src = self.readString(srcPtr, srcLen);

          let img = self.imageCache.get(src);
          if (!img) {
            img = new Image();
            img.src = src;
            self.imageCache.set(src, img);
          }

          if (img.complete) {
            entry.ctx.drawImage(img, x, y, width, height);
          } else {
            img.onload = () => {
              entry.ctx.drawImage(img, x, y, width, height);
            };
          }
          return 0;
        },

        _canvas_image_cropped(canvasId, imageId, sx, sy, sw, sh, dx, dy, dw, dh) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const asset = self.assets.get(imageId);
          if (!asset || !asset.image) return -1;
          entry.ctx.drawImage(asset.image, sx, sy, sw, sh, dx, dy, dw, dh);
          return 0;
        },

        _canvas_image_rotated(canvasId, imageId, x, y, width, height, angle) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const asset = self.assets.get(imageId);
          if (!asset || !asset.image) return -1;

          const ctx = entry.ctx;
          ctx.save();
          ctx.translate(x + width / 2, y + height / 2);
          ctx.rotate(angle * Math.PI / 180);
          ctx.drawImage(asset.image, -width / 2, -height / 2, width, height);
          ctx.restore();
          return 0;
        },

        // ====================================================================
        // TRANSFORMS
        // ====================================================================

        _canvas_save(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.save();
          return 0;
        },

        _canvas_restore(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.restore();
          return 0;
        },

        _canvas_translate(canvasId, x, y) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.translate(x, y);
          return 0;
        },

        _canvas_rotate(canvasId, angleDegrees) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.rotate(angleDegrees * Math.PI / 180);
          return 0;
        },

        _canvas_scale(canvasId, sx, sy) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.scale(sx, sy);
          return 0;
        },

        _canvas_transform(canvasId, a, b, c, d, e, f) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.transform(a, b, c, d, e, f);
          return 0;
        },

        _canvas_reset_transform(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.resetTransform();
          return 0;
        },

        // ====================================================================
        // ANIMATION
        // ====================================================================

        _canvas_request_frame(canvasId) {
          const frameId = self.nextFrameId++;
          console.log('[CanvasBridge] _canvas_request_frame called, frameId:', frameId, 'canvasId:', canvasId);

          let debugFrameCount = 0;
          const callback = (timestamp) => {
            // Calculate delta time
            self.deltaTime = Math.min((timestamp - self.lastFrameTime) / 1000, 0.1);
            self.lastFrameTime = timestamp;

            // Debug first few frames
            debugFrameCount++;
            if (debugFrameCount <= 3) {
              console.log('[CanvasBridge] Animation frame', debugFrameCount, 'wasmInstance:', !!self.wasmInstance, 'has _frame_callback:', !!(self.wasmInstance && self.wasmInstance.exports._frame_callback));
            }

            // Calculate FPS
            self.frameCount++;
            if (timestamp - self.fpsLastTime >= 1000) {
              self.fps = self.frameCount;
              self.frameCount = 0;
              self.fpsLastTime = timestamp;
            }

            // Poll gamepads
            self._pollGamepads();

            // Update camera shake
            if (self.cameraShakeDuration > 0) {
              const elapsed = (performance.now() - self.cameraShakeStart) / 1000;
              if (elapsed >= self.cameraShakeDuration) {
                self.cameraShakeDuration = 0;
                self.cameraShakeIntensity = 0;
              }
            }

            // Call WASM frame handler
            if (self.wasmInstance && self.wasmInstance.exports._frame_callback) {
              try {
                self.wasmInstance.exports._frame_callback(canvasId);
              } catch (err) {
                console.error('[CanvasBridge] Frame callback error:', err);
              }
            } else if (debugFrameCount <= 3) {
              console.log('[CanvasBridge] Skipping frame callback - wasmInstance not set');
            }

            // Clear per-frame input state
            self._clearFrameInputState();

            // Continue animation loop
            if (self.animationFrames.has(frameId)) {
              self.animationFrames.set(frameId, requestAnimationFrame(callback));
            }
          };

          self.animationFrames.set(frameId, requestAnimationFrame(callback));
          return frameId;
        },

        _canvas_cancel_frame(frameId) {
          const rafId = self.animationFrames.get(frameId);
          if (rafId) {
            cancelAnimationFrame(rafId);
            self.animationFrames.delete(frameId);
            return 0;
          }
          return -1;
        },

        _canvas_get_delta_time() {
          return self.deltaTime;
        },

        _canvas_get_time() {
          return (performance.now() - self.startTime) / 1000;
        },

        _canvas_get_fps() {
          return self.fps;
        },

        // ====================================================================
        // AUDIO - SOUND EFFECTS
        // ====================================================================

        _audio_load_sound(pathPtr, pathLen) {
          const path = self.readString(pathPtr, pathLen);
          const id = self.nextSoundId++;

          fetch(path)
            .then(response => response.arrayBuffer())
            .then(buffer => {
              if (self.audioContext) {
                return self.audioContext.decodeAudioData(buffer);
              }
              return null;
            })
            .then(audioBuffer => {
              if (audioBuffer) {
                self.sounds.set(id, {
                  buffer: audioBuffer,
                  loaded: true
                });
                console.log(`[CanvasBridge] Sound ${id} loaded: ${path}`);
              }
            })
            .catch(err => {
              console.error(`[CanvasBridge] Failed to load sound: ${path}`, err);
              self.sounds.set(id, { buffer: null, loaded: false });
            });

          self.sounds.set(id, { buffer: null, loaded: false });
          return id;
        },

        _audio_play(soundId, volume, loop) {
          const sound = self.sounds.get(soundId);
          if (!sound || !sound.buffer || !self.audioContext) return -1;

          const source = self.audioContext.createBufferSource();
          source.buffer = sound.buffer;
          source.loop = loop;

          const gainNode = self.audioContext.createGain();
          gainNode.gain.value = volume * self.masterVolume * (self.isMuted ? 0 : 1);

          source.connect(gainNode);
          gainNode.connect(self.audioContext.destination);

          const instanceId = self.nextInstanceId++;
          self.soundInstances.set(instanceId, {
            source: source,
            gainNode: gainNode,
            playing: true
          });

          source.onended = () => {
            const instance = self.soundInstances.get(instanceId);
            if (instance) instance.playing = false;
          };

          source.start(0);
          return instanceId;
        },

        _audio_stop(instanceId) {
          const instance = self.soundInstances.get(instanceId);
          if (!instance) return -1;
          try {
            instance.source.stop();
            instance.playing = false;
          } catch (e) { }
          return 0;
        },

        _audio_pause(instanceId) {
          // Web Audio API doesn't support pause, so we simulate with stop
          // For proper pause, we'd need to track playback position
          return self.env._audio_stop(instanceId);
        },

        _audio_resume(instanceId) {
          // Would require re-creating the source node
          return -1;
        },

        _audio_set_volume(instanceId, volume) {
          const instance = self.soundInstances.get(instanceId);
          if (!instance) return -1;
          instance.gainNode.gain.value = volume * self.masterVolume * (self.isMuted ? 0 : 1);
          return 0;
        },

        _audio_set_pitch(instanceId, pitch) {
          const instance = self.soundInstances.get(instanceId);
          if (!instance) return -1;
          instance.source.playbackRate.value = pitch;
          return 0;
        },

        _audio_set_pan(instanceId, pan) {
          // Would require a StereoPannerNode - skipping for simplicity
          return 0;
        },

        _audio_is_playing(instanceId) {
          const instance = self.soundInstances.get(instanceId);
          return instance ? instance.playing : false;
        },

        // ====================================================================
        // AUDIO - MUSIC
        // ====================================================================

        _audio_load_music(pathPtr, pathLen) {
          const path = self.readString(pathPtr, pathLen);
          const id = self.nextMusicId++;

          const audio = new Audio();
          audio.src = path;
          audio.preload = 'auto';

          self.music.set(id, {
            element: audio,
            loaded: false,
            path: path
          });

          audio.addEventListener('canplaythrough', () => {
            const m = self.music.get(id);
            if (m) m.loaded = true;
            console.log(`[CanvasBridge] Music ${id} loaded: ${path}`);
          });

          return id;
        },

        _audio_play_music(musicId, volume, loop) {
          const m = self.music.get(musicId);
          if (!m) return -1;

          // Stop current music
          if (self.currentMusicElement) {
            self.currentMusicElement.pause();
            self.currentMusicElement.currentTime = 0;
          }

          m.element.loop = loop;
          m.element.volume = volume * self.musicVolume * self.masterVolume * (self.isMuted ? 0 : 1);

          self.currentMusic = musicId;
          self.currentMusicElement = m.element;

          m.element.play().catch(err => {
            console.warn('[CanvasBridge] Music play failed (user interaction required):', err);
          });

          return 0;
        },

        _audio_stop_music() {
          if (self.currentMusicElement) {
            self.currentMusicElement.pause();
            self.currentMusicElement.currentTime = 0;
            self.currentMusicElement = null;
            self.currentMusic = null;
          }
          return 0;
        },

        _audio_pause_music() {
          if (self.currentMusicElement) {
            self.currentMusicElement.pause();
          }
          return 0;
        },

        _audio_resume_music() {
          if (self.currentMusicElement) {
            self.currentMusicElement.play().catch(() => { });
          }
          return 0;
        },

        _audio_set_music_volume(volume) {
          self.musicVolume = volume;
          if (self.currentMusicElement) {
            self.currentMusicElement.volume = volume * self.masterVolume * (self.isMuted ? 0 : 1);
          }
          return 0;
        },

        _audio_fade_music(targetVolume, duration) {
          if (!self.currentMusicElement) return -1;

          const startVolume = self.currentMusicElement.volume;
          const startTime = performance.now();

          const fade = () => {
            const elapsed = (performance.now() - startTime) / 1000;
            const t = Math.min(elapsed / duration, 1);
            const volume = startVolume + (targetVolume - startVolume) * t;
            if (self.currentMusicElement) {
              self.currentMusicElement.volume = volume * (self.isMuted ? 0 : 1);
            }
            if (t < 1) {
              requestAnimationFrame(fade);
            }
          };

          requestAnimationFrame(fade);
          return 0;
        },

        _audio_crossfade(musicId, volume, duration) {
          // Fade out current, fade in new
          const fadeOutPromise = new Promise(resolve => {
            if (self.currentMusicElement) {
              const startVolume = self.currentMusicElement.volume;
              const startTime = performance.now();
              const fade = () => {
                const elapsed = (performance.now() - startTime) / 1000;
                const t = Math.min(elapsed / duration, 1);
                if (self.currentMusicElement) {
                  self.currentMusicElement.volume = startVolume * (1 - t);
                }
                if (t < 1) {
                  requestAnimationFrame(fade);
                } else {
                  if (self.currentMusicElement) {
                    self.currentMusicElement.pause();
                  }
                  resolve();
                }
              };
              requestAnimationFrame(fade);
            } else {
              resolve();
            }
          });

          fadeOutPromise.then(() => {
            // Start new music with fade in
            const m = self.music.get(musicId);
            if (!m) return;

            m.element.volume = 0;
            m.element.loop = true;
            self.currentMusic = musicId;
            self.currentMusicElement = m.element;

            m.element.play().then(() => {
              const startTime = performance.now();
              const fadeIn = () => {
                const elapsed = (performance.now() - startTime) / 1000;
                const t = Math.min(elapsed / duration, 1);
                if (self.currentMusicElement) {
                  self.currentMusicElement.volume = volume * t * self.masterVolume * (self.isMuted ? 0 : 1);
                }
                if (t < 1) {
                  requestAnimationFrame(fadeIn);
                }
              };
              requestAnimationFrame(fadeIn);
            }).catch(() => { });
          });

          return 0;
        },

        // ====================================================================
        // AUDIO - MASTER CONTROL
        // ====================================================================

        _audio_set_master_volume(volume) {
          self.masterVolume = volume;
          // Update all playing sounds
          for (const [id, instance] of self.soundInstances) {
            if (instance.playing) {
              instance.gainNode.gain.value = instance.gainNode.gain.value; // Triggers recalc
            }
          }
          if (self.currentMusicElement) {
            self.currentMusicElement.volume = self.musicVolume * volume * (self.isMuted ? 0 : 1);
          }
          return 0;
        },

        _audio_mute_all() {
          self.isMuted = true;
          for (const [id, instance] of self.soundInstances) {
            instance.gainNode.gain.value = 0;
          }
          if (self.currentMusicElement) {
            self.currentMusicElement.volume = 0;
          }
          return 0;
        },

        _audio_unmute_all() {
          self.isMuted = false;
          if (self.currentMusicElement) {
            self.currentMusicElement.volume = self.musicVolume * self.masterVolume;
          }
          return 0;
        },

        // ====================================================================
        // SPRITES
        // ====================================================================

        _sprite_load_sheet(pathPtr, pathLen, frameWidth, frameHeight) {
          const path = self.readString(pathPtr, pathLen);
          const id = self.nextSpriteId++;

          const img = new Image();
          img.src = path;

          self.spriteSheets.set(id, {
            image: img,
            frameWidth: frameWidth,
            frameHeight: frameHeight,
            loaded: false,
            framesPerRow: 0,
            totalFrames: 0
          });

          img.onload = () => {
            const sheet = self.spriteSheets.get(id);
            if (sheet) {
              sheet.loaded = true;
              sheet.framesPerRow = Math.floor(img.width / frameWidth);
              const rows = Math.floor(img.height / frameHeight);
              sheet.totalFrames = sheet.framesPerRow * rows;
              console.log(`[CanvasBridge] Sprite sheet ${id} loaded: ${path} (${sheet.totalFrames} frames)`);
            }
          };

          return id;
        },

        _sprite_draw(canvasId, sheetId, frameIndex, x, y) {
          const entry = self.canvases.get(canvasId);
          const sheet = self.spriteSheets.get(sheetId);
          if (!entry || !sheet || !sheet.loaded) return -1;

          const col = frameIndex % sheet.framesPerRow;
          const row = Math.floor(frameIndex / sheet.framesPerRow);
          const sx = col * sheet.frameWidth;
          const sy = row * sheet.frameHeight;

          entry.ctx.drawImage(
            sheet.image,
            sx, sy, sheet.frameWidth, sheet.frameHeight,
            x, y, sheet.frameWidth, sheet.frameHeight
          );
          return 0;
        },

        _sprite_draw_scaled(canvasId, sheetId, frameIndex, x, y, scaleX, scaleY) {
          const entry = self.canvases.get(canvasId);
          const sheet = self.spriteSheets.get(sheetId);
          if (!entry || !sheet || !sheet.loaded) return -1;

          const col = frameIndex % sheet.framesPerRow;
          const row = Math.floor(frameIndex / sheet.framesPerRow);
          const sx = col * sheet.frameWidth;
          const sy = row * sheet.frameHeight;
          const dw = sheet.frameWidth * scaleX;
          const dh = sheet.frameHeight * scaleY;

          entry.ctx.drawImage(
            sheet.image,
            sx, sy, sheet.frameWidth, sheet.frameHeight,
            x, y, dw, dh
          );
          return 0;
        },

        _sprite_draw_rotated(canvasId, sheetId, frameIndex, x, y, angle) {
          const entry = self.canvases.get(canvasId);
          const sheet = self.spriteSheets.get(sheetId);
          if (!entry || !sheet || !sheet.loaded) return -1;

          const col = frameIndex % sheet.framesPerRow;
          const row = Math.floor(frameIndex / sheet.framesPerRow);
          const sx = col * sheet.frameWidth;
          const sy = row * sheet.frameHeight;

          const ctx = entry.ctx;
          ctx.save();
          ctx.translate(x + sheet.frameWidth / 2, y + sheet.frameHeight / 2);
          ctx.rotate(angle * Math.PI / 180);
          ctx.drawImage(
            sheet.image,
            sx, sy, sheet.frameWidth, sheet.frameHeight,
            -sheet.frameWidth / 2, -sheet.frameHeight / 2, sheet.frameWidth, sheet.frameHeight
          );
          ctx.restore();
          return 0;
        },

        _sprite_draw_transformed(canvasId, sheetId, frameIndex, x, y, scaleX, scaleY, angle) {
          const entry = self.canvases.get(canvasId);
          const sheet = self.spriteSheets.get(sheetId);
          if (!entry || !sheet || !sheet.loaded) return -1;

          const col = frameIndex % sheet.framesPerRow;
          const row = Math.floor(frameIndex / sheet.framesPerRow);
          const sx = col * sheet.frameWidth;
          const sy = row * sheet.frameHeight;
          const dw = sheet.frameWidth * scaleX;
          const dh = sheet.frameHeight * scaleY;

          const ctx = entry.ctx;
          ctx.save();
          ctx.translate(x + dw / 2, y + dh / 2);
          ctx.rotate(angle * Math.PI / 180);
          ctx.drawImage(
            sheet.image,
            sx, sy, sheet.frameWidth, sheet.frameHeight,
            -dw / 2, -dh / 2, dw, dh
          );
          ctx.restore();
          return 0;
        },

        _sprite_get_frame_count(sheetId) {
          const sheet = self.spriteSheets.get(sheetId);
          return sheet ? sheet.totalFrames : 0;
        },

        _sprite_get_frame_width(sheetId) {
          const sheet = self.spriteSheets.get(sheetId);
          return sheet ? sheet.frameWidth : 0;
        },

        _sprite_get_frame_height(sheetId) {
          const sheet = self.spriteSheets.get(sheetId);
          return sheet ? sheet.frameHeight : 0;
        },

        // ====================================================================
        // INPUT - MOUSE
        // ====================================================================

        _input_mouse_x() {
          return self.mouseX;
        },

        _input_mouse_y() {
          return self.mouseY;
        },

        _input_mouse_pressed(button) {
          return self.mouseButtons[button] || false;
        },

        _input_mouse_just_pressed(button) {
          return self.mouseButtonsJustPressed[button] || false;
        },

        _input_mouse_just_released(button) {
          return self.mouseButtonsJustReleased[button] || false;
        },

        _input_mouse_wheel_x() {
          return self.mouseWheelX;
        },

        _input_mouse_wheel_y() {
          return self.mouseWheelY;
        },

        // ====================================================================
        // INPUT - KEYBOARD
        // ====================================================================

        _input_key_down(keyPtr, keyLen) {
          const key = self.readString(keyPtr, keyLen);
          return self.keysDown.has(key) ? 1 : 0;
        },

        _input_key_just_pressed(keyPtr, keyLen) {
          const key = self.readString(keyPtr, keyLen);
          return self.keysJustPressed.has(key) ? 1 : 0;
        },

        _input_key_just_released(keyPtr, keyLen) {
          const key = self.readString(keyPtr, keyLen);
          return self.keysJustReleased.has(key) ? 1 : 0;
        },

        _input_get_last_key() {
          // Would need to write to WASM memory - return 0 for now
          return 0;
        },

        _input_get_text_input() {
          // Would need to write to WASM memory - return 0 for now
          return 0;
        },

        // ====================================================================
        // INPUT - TOUCH
        // ====================================================================

        _input_touch_count() {
          return self.touches.length;
        },

        _input_touch_x(index) {
          return self.touches[index] ? self.touches[index].x : 0;
        },

        _input_touch_y(index) {
          return self.touches[index] ? self.touches[index].y : 0;
        },

        _input_touch_id(index) {
          return self.touches[index] ? self.touches[index].id : -1;
        },

        _input_touch_started() {
          return self.touchStarted;
        },

        _input_touch_ended() {
          return self.touchEnded;
        },

        // ====================================================================
        // INPUT - GAMEPAD
        // ====================================================================

        _input_gamepad_connected(index) {
          const gp = self.gamepads.get(index);
          return gp ? gp.connected : false;
        },

        _input_gamepad_button(gamepadIndex, buttonIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp && gp.buttons[buttonIndex] ? true : false;
        },

        _input_gamepad_button_just_pressed(gamepadIndex, buttonIndex) {
          const jp = self.gamepadButtonsJustPressed.get(gamepadIndex);
          return jp && jp[buttonIndex] ? true : false;
        },

        _input_gamepad_axis(gamepadIndex, axisIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp && gp.axes[axisIndex] !== undefined ? gp.axes[axisIndex] : 0;
        },

        _input_gamepad_left_stick_x(gamepadIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp ? (gp.axes[0] || 0) : 0;
        },

        _input_gamepad_left_stick_y(gamepadIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp ? (gp.axes[1] || 0) : 0;
        },

        _input_gamepad_right_stick_x(gamepadIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp ? (gp.axes[2] || 0) : 0;
        },

        _input_gamepad_right_stick_y(gamepadIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp ? (gp.axes[3] || 0) : 0;
        },

        _input_gamepad_left_trigger(gamepadIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp && gp.buttons[6] ? gp.buttons[6].value || (gp.buttons[6] ? 1 : 0) : 0;
        },

        _input_gamepad_right_trigger(gamepadIndex) {
          const gp = self.gamepads.get(gamepadIndex);
          return gp && gp.buttons[7] ? gp.buttons[7].value || (gp.buttons[7] ? 1 : 0) : 0;
        },

        _input_gamepad_vibrate(gamepadIndex, intensity, duration) {
          const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
          const gp = gamepads[gamepadIndex];
          if (gp && gp.vibrationActuator) {
            gp.vibrationActuator.playEffect('dual-rumble', {
              duration: duration * 1000,
              strongMagnitude: intensity,
              weakMagnitude: intensity
            }).catch(() => { });
          }
          return 0;
        },

        // ====================================================================
        // COLLISION DETECTION
        // ====================================================================

        _collision_point_rect(px, py, rx, ry, rw, rh) {
          return (px >= rx && px <= rx + rw && py >= ry && py <= ry + rh) ? 1 : 0;
        },

        _collision_point_circle(px, py, cx, cy, radius) {
          const dx = px - cx;
          const dy = py - cy;
          return (dx * dx + dy * dy <= radius * radius) ? 1 : 0;
        },

        _collision_circle_circle(x1, y1, r1, x2, y2, r2) {
          const dx = x2 - x1;
          const dy = y2 - y1;
          const dist = Math.sqrt(dx * dx + dy * dy);
          return (dist <= r1 + r2) ? 1 : 0;
        },

        _collision_rect_rect(x1, y1, w1, h1, x2, y2, w2, h2) {
          return (x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2) ? 1 : 0;
        },

        _collision_circle_rect(cx, cy, cr, rx, ry, rw, rh) {
          const closestX = Math.max(rx, Math.min(cx, rx + rw));
          const closestY = Math.max(ry, Math.min(cy, ry + rh));
          const dx = cx - closestX;
          const dy = cy - closestY;
          return (dx * dx + dy * dy <= cr * cr) ? 1 : 0;
        },

        _collision_line_line(x1, y1, x2, y2, x3, y3, x4, y4) {
          const denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
          if (Math.abs(denom) < 0.0001) return 0;

          const ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
          const ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

          return (ua >= 0 && ua <= 1 && ub >= 0 && ub <= 1) ? 1 : 0;
        },

        _collision_line_circle(x1, y1, x2, y2, cx, cy, r) {
          const dx = x2 - x1;
          const dy = y2 - y1;
          const fx = x1 - cx;
          const fy = y1 - cy;

          const a = dx * dx + dy * dy;
          const b = 2 * (fx * dx + fy * dy);
          const c = fx * fx + fy * fy - r * r;

          let discriminant = b * b - 4 * a * c;
          if (discriminant < 0) return 0;

          discriminant = Math.sqrt(discriminant);
          const t1 = (-b - discriminant) / (2 * a);
          const t2 = (-b + discriminant) / (2 * a);

          return ((t1 >= 0 && t1 <= 1) || (t2 >= 0 && t2 <= 1)) ? 1 : 0;
        },

        _collision_line_rect(x1, y1, x2, y2, rx, ry, rw, rh) {
          // Check all 4 edges
          const left = self.env._collision_line_line(x1, y1, x2, y2, rx, ry, rx, ry + rh);
          const right = self.env._collision_line_line(x1, y1, x2, y2, rx + rw, ry, rx + rw, ry + rh);
          const top = self.env._collision_line_line(x1, y1, x2, y2, rx, ry, rx + rw, ry);
          const bottom = self.env._collision_line_line(x1, y1, x2, y2, rx, ry + rh, rx + rw, ry + rh);
          return (left || right || top || bottom) ? 1 : 0;
        },

        _collision_circle_circle_overlap(x1, y1, r1, x2, y2, r2) {
          const dx = x2 - x1;
          const dy = y2 - y1;
          const dist = Math.sqrt(dx * dx + dy * dy);
          return (r1 + r2) - dist;
        },

        _collision_rect_rect_overlap_x(x1, y1, w1, h1, x2, y2, w2, h2) {
          const overlapLeft = (x1 + w1) - x2;
          const overlapRight = (x2 + w2) - x1;
          return Math.min(overlapLeft, overlapRight);
        },

        _collision_rect_rect_overlap_y(x1, y1, w1, h1, x2, y2, w2, h2) {
          const overlapTop = (y1 + h1) - y2;
          const overlapBottom = (y2 + h2) - y1;
          return Math.min(overlapTop, overlapBottom);
        },

        _collision_raycast_circle(ox, oy, dx, dy, cx, cy, r) {
          const fx = ox - cx;
          const fy = oy - cy;

          const a = dx * dx + dy * dy;
          const b = 2 * (fx * dx + fy * dy);
          const c = fx * fx + fy * fy - r * r;

          let discriminant = b * b - 4 * a * c;
          if (discriminant < 0) return -1;

          discriminant = Math.sqrt(discriminant);
          const t1 = (-b - discriminant) / (2 * a);
          const t2 = (-b + discriminant) / (2 * a);

          if (t1 >= 0) return t1;
          if (t2 >= 0) return t2;
          return -1;
        },

        _collision_raycast_rect(ox, oy, dx, dy, rx, ry, rw, rh) {
          let tmin = -Infinity;
          let tmax = Infinity;

          if (dx !== 0) {
            const t1 = (rx - ox) / dx;
            const t2 = (rx + rw - ox) / dx;
            tmin = Math.max(tmin, Math.min(t1, t2));
            tmax = Math.min(tmax, Math.max(t1, t2));
          } else if (ox < rx || ox > rx + rw) {
            return -1;
          }

          if (dy !== 0) {
            const t1 = (ry - oy) / dy;
            const t2 = (ry + rh - oy) / dy;
            tmin = Math.max(tmin, Math.min(t1, t2));
            tmax = Math.min(tmax, Math.max(t1, t2));
          } else if (oy < ry || oy > ry + rh) {
            return -1;
          }

          if (tmax >= tmin && tmax >= 0) {
            return tmin >= 0 ? tmin : tmax;
          }
          return -1;
        },

        // ====================================================================
        // ASSET MANAGEMENT
        // ====================================================================

        _asset_load_image(pathPtr, pathLen) {
          const path = self.readString(pathPtr, pathLen);
          const id = self.assets.size + 1;

          const img = new Image();
          img.src = path;

          self.assets.set(id, {
            type: 'image',
            path: path,
            image: img,
            loaded: false
          });

          img.onload = () => {
            const asset = self.assets.get(id);
            if (asset) asset.loaded = true;
          };

          return id;
        },

        _asset_load_sound(pathPtr, pathLen) {
          return self.env._audio_load_sound(pathPtr, pathLen);
        },

        _asset_load_music(pathPtr, pathLen) {
          return self.env._audio_load_music(pathPtr, pathLen);
        },

        _asset_queue(pathPtr, pathLen) {
          const path = self.readString(pathPtr, pathLen);
          self.assetQueue.push(path);
          self.assetsTotal = self.assetQueue.length;
          return self.assetQueue.length - 1;
        },

        _asset_load_all() {
          self.assetsLoaded = 0;

          const promises = self.assetQueue.map((path, index) => {
            return new Promise((resolve) => {
              if (path.match(/\.(png|jpg|jpeg|gif|webp)$/i)) {
                const img = new Image();
                img.src = path;
                img.onload = () => {
                  self.assetsLoaded++;
                  self.assets.set(path, { type: 'image', image: img, loaded: true });
                  resolve();
                };
                img.onerror = () => {
                  self.assetsLoaded++;
                  resolve();
                };
              } else if (path.match(/\.(wav|mp3|ogg)$/i)) {
                fetch(path)
                  .then(r => r.arrayBuffer())
                  .then(buffer => {
                    if (self.audioContext) {
                      return self.audioContext.decodeAudioData(buffer);
                    }
                    return null;
                  })
                  .then(audioBuffer => {
                    self.assetsLoaded++;
                    if (audioBuffer) {
                      self.assets.set(path, { type: 'sound', buffer: audioBuffer, loaded: true });
                    }
                    resolve();
                  })
                  .catch(() => {
                    self.assetsLoaded++;
                    resolve();
                  });
              } else {
                self.assetsLoaded++;
                resolve();
              }
            });
          });

          Promise.all(promises).then(() => {
            console.log(`[CanvasBridge] All ${self.assetsTotal} assets loaded`);
          });

          return 0;
        },

        _asset_get_progress() {
          return self.assetsTotal > 0 ? self.assetsLoaded / self.assetsTotal : 1;
        },

        _asset_all_loaded() {
          return self.assetsLoaded >= self.assetsTotal;
        },

        _asset_is_loaded(assetId) {
          const asset = self.assets.get(assetId);
          return asset ? asset.loaded : false;
        },

        _asset_get(pathPtr, pathLen) {
          const path = self.readString(pathPtr, pathLen);
          for (const [id, asset] of self.assets) {
            if (asset.path === path) return id;
          }
          return -1;
        },

        _asset_unload(assetId) {
          self.assets.delete(assetId);
          return 0;
        },

        _asset_unload_all() {
          self.assets.clear();
          self.assetQueue = [];
          self.assetsLoaded = 0;
          self.assetsTotal = 0;
          return 0;
        },

        // ====================================================================
        // CAMERA & VIEWPORT
        // ====================================================================

        _camera_set_position(x, y) {
          self.cameraX = x;
          self.cameraY = y;
          return 0;
        },

        _camera_get_x() {
          return self.cameraX;
        },

        _camera_get_y() {
          return self.cameraY;
        },

        _camera_set_zoom(zoom) {
          self.cameraZoom = zoom;
          return 0;
        },

        _camera_get_zoom() {
          return self.cameraZoom;
        },

        _camera_set_rotation(degrees) {
          self.cameraRotation = degrees;
          return 0;
        },

        _camera_apply(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;

          const ctx = entry.ctx;
          ctx.save();

          // Apply shake
          let shakeX = 0, shakeY = 0;
          if (self.cameraShakeDuration > 0) {
            const elapsed = (performance.now() - self.cameraShakeStart) / 1000;
            if (elapsed < self.cameraShakeDuration) {
              const decay = 1 - elapsed / self.cameraShakeDuration;
              shakeX = (Math.random() - 0.5) * 2 * self.cameraShakeIntensity * decay;
              shakeY = (Math.random() - 0.5) * 2 * self.cameraShakeIntensity * decay;
            }
          }

          // Center on canvas
          ctx.translate(entry.width / 2, entry.height / 2);

          // Apply rotation
          if (self.cameraRotation !== 0) {
            ctx.rotate(self.cameraRotation * Math.PI / 180);
          }

          // Apply zoom
          ctx.scale(self.cameraZoom, self.cameraZoom);

          // Apply position (inverted for camera effect)
          ctx.translate(-self.cameraX + shakeX, -self.cameraY + shakeY);

          return 0;
        },

        _camera_reset(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.restore();
          return 0;
        },

        _camera_screen_to_world_x(screenX) {
          const entry = self.canvases.values().next().value;
          if (!entry) return screenX;
          return (screenX - entry.width / 2) / self.cameraZoom + self.cameraX;
        },

        _camera_screen_to_world_y(screenY) {
          const entry = self.canvases.values().next().value;
          if (!entry) return screenY;
          return (screenY - entry.height / 2) / self.cameraZoom + self.cameraY;
        },

        _camera_world_to_screen_x(worldX) {
          const entry = self.canvases.values().next().value;
          if (!entry) return worldX;
          return (worldX - self.cameraX) * self.cameraZoom + entry.width / 2;
        },

        _camera_world_to_screen_y(worldY) {
          const entry = self.canvases.values().next().value;
          if (!entry) return worldY;
          return (worldY - self.cameraY) * self.cameraZoom + entry.height / 2;
        },

        _camera_shake(intensity, duration) {
          self.cameraShakeIntensity = intensity;
          self.cameraShakeDuration = duration;
          self.cameraShakeStart = performance.now();
          return 0;
        },

        // ====================================================================
        // GRADIENTS
        // ====================================================================

        _gradient_create_linear(x1, y1, x2, y2) {
          const id = self.nextGradientId++;
          self.gradients.set(id, {
            type: 'linear',
            x1, y1, x2, y2,
            stops: []
          });
          return id;
        },

        _gradient_create_radial(cx, cy, innerRadius, outerRadius) {
          const id = self.nextGradientId++;
          self.gradients.set(id, {
            type: 'radial',
            cx, cy, innerRadius, outerRadius,
            stops: []
          });
          return id;
        },

        _gradient_add_stop(gradientId, offset, colorPtr, colorLen) {
          const gradient = self.gradients.get(gradientId);
          if (!gradient) return -1;
          const color = self.readString(colorPtr, colorLen);
          gradient.stops.push({ offset, color });
          return 0;
        },

        _canvas_set_fill_gradient(canvasId, gradientId) {
          const entry = self.canvases.get(canvasId);
          const gradientDef = self.gradients.get(gradientId);
          if (!entry || !gradientDef) return -1;

          const ctx = entry.ctx;
          let gradient;

          if (gradientDef.type === 'linear') {
            gradient = ctx.createLinearGradient(gradientDef.x1, gradientDef.y1, gradientDef.x2, gradientDef.y2);
          } else {
            gradient = ctx.createRadialGradient(
              gradientDef.cx, gradientDef.cy, gradientDef.innerRadius,
              gradientDef.cx, gradientDef.cy, gradientDef.outerRadius
            );
          }

          for (const stop of gradientDef.stops) {
            gradient.addColorStop(stop.offset, stop.color);
          }

          ctx.fillStyle = gradient;
          return 0;
        },

        _canvas_set_stroke_gradient(canvasId, gradientId) {
          const entry = self.canvases.get(canvasId);
          const gradientDef = self.gradients.get(gradientId);
          if (!entry || !gradientDef) return -1;

          const ctx = entry.ctx;
          let gradient;

          if (gradientDef.type === 'linear') {
            gradient = ctx.createLinearGradient(gradientDef.x1, gradientDef.y1, gradientDef.x2, gradientDef.y2);
          } else {
            gradient = ctx.createRadialGradient(
              gradientDef.cx, gradientDef.cy, gradientDef.innerRadius,
              gradientDef.cx, gradientDef.cy, gradientDef.outerRadius
            );
          }

          for (const stop of gradientDef.stops) {
            gradient.addColorStop(stop.offset, stop.color);
          }

          ctx.strokeStyle = gradient;
          return 0;
        },

        // ====================================================================
        // PATHS
        // ====================================================================

        _path_begin(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.beginPath();
          return 0;
        },

        _path_move_to(canvasId, x, y) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.moveTo(x, y);
          return 0;
        },

        _path_line_to(canvasId, x, y) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.lineTo(x, y);
          return 0;
        },

        _path_quadratic_to(canvasId, cpx, cpy, x, y) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.quadraticCurveTo(cpx, cpy, x, y);
          return 0;
        },

        _path_bezier_to(canvasId, cp1x, cp1y, cp2x, cp2y, x, y) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.bezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y);
          return 0;
        },

        _path_arc(canvasId, x, y, radius, startAngle, endAngle) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.arc(x, y, radius, startAngle * Math.PI / 180, endAngle * Math.PI / 180);
          return 0;
        },

        _path_arc_to(canvasId, x1, y1, x2, y2, radius) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.arcTo(x1, y1, x2, y2, radius);
          return 0;
        },

        _path_close(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.closePath();
          return 0;
        },

        _path_fill(canvasId, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          entry.ctx.fillStyle = color;
          entry.ctx.fill();
          return 0;
        },

        _path_stroke(canvasId, colorPtr, colorLen, lineWidth) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          entry.ctx.strokeStyle = color;
          entry.ctx.lineWidth = lineWidth;
          entry.ctx.stroke();
          return 0;
        },

        // ====================================================================
        // ADVANCED RENDERING
        // ====================================================================

        _canvas_set_blend_mode(canvasId, modePtr, modeLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const mode = self.readString(modePtr, modeLen);
          entry.ctx.globalCompositeOperation = mode;
          return 0;
        },

        _canvas_set_alpha(canvasId, alpha) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.globalAlpha = alpha;
          return 0;
        },

        _canvas_set_shadow(canvasId, blur, offsetX, offsetY, colorPtr, colorLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const color = self.readString(colorPtr, colorLen);
          entry.ctx.shadowBlur = blur;
          entry.ctx.shadowOffsetX = offsetX;
          entry.ctx.shadowOffsetY = offsetY;
          entry.ctx.shadowColor = color;
          return 0;
        },

        _canvas_clear_shadow(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.shadowBlur = 0;
          entry.ctx.shadowOffsetX = 0;
          entry.ctx.shadowOffsetY = 0;
          entry.ctx.shadowColor = 'transparent';
          return 0;
        },

        _canvas_set_line_cap(canvasId, capPtr, capLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const cap = self.readString(capPtr, capLen);
          entry.ctx.lineCap = cap;
          return 0;
        },

        _canvas_set_line_join(canvasId, joinPtr, joinLen) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          const join = self.readString(joinPtr, joinLen);
          entry.ctx.lineJoin = join;
          return 0;
        },

        _canvas_set_line_dash(canvasId, dashLength, gapLength) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.setLineDash([dashLength, gapLength]);
          return 0;
        },

        _canvas_clear_line_dash(canvasId) {
          const entry = self.canvases.get(canvasId);
          if (!entry) return -1;
          entry.ctx.setLineDash([]);
          return 0;
        },

        // ====================================================================
        // EASING FUNCTIONS
        // ====================================================================

        _ease_linear(t) {
          return t;
        },

        _ease_in_quad(t) {
          return t * t;
        },

        _ease_out_quad(t) {
          return t * (2 - t);
        },

        _ease_in_out_quad(t) {
          return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
        },

        _ease_in_cubic(t) {
          return t * t * t;
        },

        _ease_out_cubic(t) {
          return (--t) * t * t + 1;
        },

        _ease_in_out_cubic(t) {
          return t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1;
        },

        _ease_in_sine(t) {
          return 1 - Math.cos(t * Math.PI / 2);
        },

        _ease_out_sine(t) {
          return Math.sin(t * Math.PI / 2);
        },

        _ease_in_out_sine(t) {
          return -(Math.cos(Math.PI * t) - 1) / 2;
        },

        _ease_in_expo(t) {
          return t === 0 ? 0 : Math.pow(2, 10 * t - 10);
        },

        _ease_out_expo(t) {
          return t === 1 ? 1 : 1 - Math.pow(2, -10 * t);
        },

        _ease_in_out_expo(t) {
          if (t === 0) return 0;
          if (t === 1) return 1;
          return t < 0.5
            ? Math.pow(2, 20 * t - 10) / 2
            : (2 - Math.pow(2, -20 * t + 10)) / 2;
        },

        _ease_in_elastic(t) {
          const c4 = (2 * Math.PI) / 3;
          return t === 0 ? 0 : t === 1 ? 1 : -Math.pow(2, 10 * t - 10) * Math.sin((t * 10 - 10.75) * c4);
        },

        _ease_out_elastic(t) {
          const c4 = (2 * Math.PI) / 3;
          return t === 0 ? 0 : t === 1 ? 1 : Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * c4) + 1;
        },

        _ease_in_out_elastic(t) {
          const c5 = (2 * Math.PI) / 4.5;
          return t === 0 ? 0 : t === 1 ? 1 : t < 0.5
            ? -(Math.pow(2, 20 * t - 10) * Math.sin((20 * t - 11.125) * c5)) / 2
            : (Math.pow(2, -20 * t + 10) * Math.sin((20 * t - 11.125) * c5)) / 2 + 1;
        },

        _ease_in_bounce(t) {
          return 1 - self.env._ease_out_bounce(1 - t);
        },

        _ease_out_bounce(t) {
          const n1 = 7.5625;
          const d1 = 2.75;
          if (t < 1 / d1) {
            return n1 * t * t;
          } else if (t < 2 / d1) {
            return n1 * (t -= 1.5 / d1) * t + 0.75;
          } else if (t < 2.5 / d1) {
            return n1 * (t -= 2.25 / d1) * t + 0.9375;
          } else {
            return n1 * (t -= 2.625 / d1) * t + 0.984375;
          }
        },

        _ease_in_out_bounce(t) {
          return t < 0.5
            ? (1 - self.env._ease_out_bounce(1 - 2 * t)) / 2
            : (1 + self.env._ease_out_bounce(2 * t - 1)) / 2;
        },

        _ease_in_back(t) {
          const c1 = 1.70158;
          const c3 = c1 + 1;
          return c3 * t * t * t - c1 * t * t;
        },

        _ease_out_back(t) {
          const c1 = 1.70158;
          const c3 = c1 + 1;
          return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
        },

        _ease_in_out_back(t) {
          const c1 = 1.70158;
          const c2 = c1 * 1.525;
          return t < 0.5
            ? (Math.pow(2 * t, 2) * ((c2 + 1) * 2 * t - c2)) / 2
            : (Math.pow(2 * t - 2, 2) * ((c2 + 1) * (t * 2 - 2) + c2) + 2) / 2;
        },

        // ====================================================================
        // SCENE MANAGEMENT
        // ====================================================================

        _scene_get_current() {
          // Would need to write to WASM memory - return 0 for now
          return 0;
        },

        _scene_change(namePtr, nameLen) {
          const name = self.readString(namePtr, nameLen);
          self.currentScene = name;
          self.sceneStack = [name];
          console.log(`[CanvasBridge] Scene changed to: ${name}`);
          return 0;
        },

        _scene_push(namePtr, nameLen) {
          const name = self.readString(namePtr, nameLen);
          self.sceneStack.push(name);
          self.currentScene = name;
          console.log(`[CanvasBridge] Scene pushed: ${name}`);
          return 0;
        },

        _scene_pop() {
          if (self.sceneStack.length > 1) {
            self.sceneStack.pop();
            self.currentScene = self.sceneStack[self.sceneStack.length - 1];
            console.log(`[CanvasBridge] Scene popped, current: ${self.currentScene}`);
            return 0;
          }
          return -1;
        },

        // ====================================================================
        // LEGACY COMPATIBILITY
        // ====================================================================

        _canvas_on_pointer(canvasId, handlerIdx) {
          // Legacy - not used with new input system
          return 0;
        },

        _canvas_on_key(canvasId, handlerIdx) {
          // Legacy - not used with new input system
          return 0;
        },

        _canvas_pointer_x() {
          return self.mouseX;
        },

        _canvas_pointer_y() {
          return self.mouseY;
        },

        // Physics helpers (legacy, can be removed)
        _physics_update() { return 0; },
        _physics_get_x() { return 0; },
        _physics_get_y() { return 0; },
        _physics_get_vel_x() { return 0; },
        _physics_get_vel_y() { return 0; }
      }
    };
  }

  // ==========================================================================
  // WASM INSTANCE SETUP
  // ==========================================================================

  setWasmInstance(instance) {
    this.wasmInstance = instance;
  }

  // ==========================================================================
  // CLEANUP
  // ==========================================================================

  destroy() {
    // Cancel all animation frames
    for (const [frameId, rafId] of this.animationFrames) {
      cancelAnimationFrame(rafId);
    }
    this.animationFrames.clear();

    // Stop all sounds
    for (const [id, instance] of this.soundInstances) {
      try { instance.source.stop(); } catch (e) { }
    }
    this.soundInstances.clear();

    // Stop music
    if (this.currentMusicElement) {
      this.currentMusicElement.pause();
      this.currentMusicElement = null;
    }

    // Remove all canvas elements
    for (const [id, entry] of this.canvases) {
      entry.element.remove();
    }
    this.canvases.clear();

    console.log('[CanvasBridge] Destroyed');
  }
}

// =============================================================================
// EXPORTS
// =============================================================================

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { CanvasBridge };
}

if (typeof window !== 'undefined') {
  window.CanvasBridge = CanvasBridge;
}
