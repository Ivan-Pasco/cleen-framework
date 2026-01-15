/**
 * Clean Canvas WASM Loader
 * Version: 2.0.0
 *
 * Loads and runs Clean Language WASM modules with canvas bridge support.
 *
 * Usage:
 *   <script src="canvas-bridge.js"></script>
 *   <script src="loader.js"></script>
 *   <script>
 *     CleanCanvas.load('app.wasm').then(app => {
 *       app.start();
 *     });
 *   </script>
 */

(function() {
  'use strict';

  // String buffer for WASM string returns
  const STRING_BUFFER_SIZE = 65536;
  const STRING_BUFFER_OFFSET = 1024 * 1024; // 1MB offset

  /**
   * Clean Canvas Application Loader
   */
  class CleanCanvasLoader {
    constructor() {
      this.bridge = new CanvasBridge();
      this.memory = null;
      this.instance = null;
      this.stringAllocOffset = STRING_BUFFER_OFFSET;
    }

    /**
     * Load a WASM module
     * @param {string} url - URL to the .wasm file
     * @param {Object} options - Load options
     * @returns {Promise<CleanCanvasApp>} Application instance
     */
    async load(url, options = {}) {
      console.log(`[CleanCanvas] Loading ${url}...`);

      // Create memory
      const memoryPages = options.memoryPages || 256; // 16MB default
      this.memory = new WebAssembly.Memory({
        initial: memoryPages,
        maximum: memoryPages * 4
      });

      // Get bridge imports
      const bridgeImports = this.bridge.getImports(this.memory);

      // Create standard library imports
      const stdlibImports = this.createStdlibImports();

      // Merge all imports
      const imports = {
        env: {
          memory: this.memory,
          ...bridgeImports.env,
          ...stdlibImports
        }
      };

      try {
        // Fetch and compile WASM
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to fetch ${url}: ${response.status}`);
        }

        const wasmBuffer = await response.arrayBuffer();
        const { instance } = await WebAssembly.instantiate(wasmBuffer, imports);

        this.instance = instance;
        this.bridge.setWasmInstance(instance);

        console.log('[CleanCanvas] WASM loaded successfully');
        console.log('[CleanCanvas] Exports:', Object.keys(instance.exports));

        return new CleanCanvasApp(this);
      } catch (err) {
        console.error('[CleanCanvas] Failed to load WASM:', err);
        throw err;
      }
    }

    /**
     * Create standard library imports (print, string operations, etc.)
     */
    createStdlibImports() {
      const self = this;

      return {
        // ================================================================
        // CONSOLE / PRINT
        // ================================================================

        _print(ptr, len) {
          const str = self.readString(ptr, len);
          console.log(str);
          return 0;
        },

        _print_i32(value) {
          console.log(value);
          return 0;
        },

        _print_f64(value) {
          console.log(value);
          return 0;
        },

        _printl(ptr, len) {
          const str = self.readString(ptr, len);
          console.log(str);
          return 0;
        },

        _console_log(ptr, len) {
          const str = self.readString(ptr, len);
          console.log(str);
          return 0;
        },

        _console_info(ptr, len) {
          const str = self.readString(ptr, len);
          console.info(str);
          return 0;
        },

        _console_warn(ptr, len) {
          const str = self.readString(ptr, len);
          console.warn(str);
          return 0;
        },

        _console_error(ptr, len) {
          const str = self.readString(ptr, len);
          console.error(str);
          return 0;
        },

        // ================================================================
        // STRING ALLOCATION
        // ================================================================

        _alloc_string(len) {
          const ptr = self.stringAllocOffset;
          self.stringAllocOffset += len + 8; // 8 byte alignment
          return ptr;
        },

        _string_concat(ptr1, len1, ptr2, len2, outPtr) {
          const str1 = self.readString(ptr1, len1);
          const str2 = self.readString(ptr2, len2);
          const result = str1 + str2;
          return self.writeString(result, outPtr);
        },

        _string_length(ptr, len) {
          return len;
        },

        // ================================================================
        // MATH FUNCTIONS
        // ================================================================

        _math_sin(x) { return Math.sin(x); },
        _math_cos(x) { return Math.cos(x); },
        _math_tan(x) { return Math.tan(x); },
        _math_asin(x) { return Math.asin(x); },
        _math_acos(x) { return Math.acos(x); },
        _math_atan(x) { return Math.atan(x); },
        _math_atan2(y, x) { return Math.atan2(y, x); },
        _math_sinh(x) { return Math.sinh(x); },
        _math_cosh(x) { return Math.cosh(x); },
        _math_tanh(x) { return Math.tanh(x); },
        _math_sqrt(x) { return Math.sqrt(x); },
        _math_cbrt(x) { return Math.cbrt(x); },
        _math_pow(x, y) { return Math.pow(x, y); },
        _math_exp(x) { return Math.exp(x); },
        _math_log(x) { return Math.log(x); },
        _math_log10(x) { return Math.log10(x); },
        _math_log2(x) { return Math.log2(x); },
        _math_abs(x) { return Math.abs(x); },
        _math_floor(x) { return Math.floor(x); },
        _math_ceil(x) { return Math.ceil(x); },
        _math_round(x) { return Math.round(x); },
        _math_trunc(x) { return Math.trunc(x); },
        _math_sign(x) { return Math.sign(x); },
        _math_min(x, y) { return Math.min(x, y); },
        _math_max(x, y) { return Math.max(x, y); },
        _math_clamp(x, min, max) { return Math.max(min, Math.min(max, x)); },
        _math_lerp(a, b, t) { return a + (b - a) * t; },
        _math_random() { return Math.random(); },
        _math_random_range(min, max) { return Math.random() * (max - min) + min; },
        _math_random_int(min, max) { return Math.floor(Math.random() * (max - min + 1)) + min; },

        // Constants
        _math_pi() { return Math.PI; },
        _math_e() { return Math.E; },

        // ================================================================
        // TIME FUNCTIONS
        // ================================================================

        _time_now() {
          return Date.now();
        },

        _time_now_seconds() {
          return Date.now() / 1000;
        },

        _time_performance_now() {
          return performance.now();
        },

        // ================================================================
        // LOGGING
        // ================================================================

        _log_info(ptr, len) {
          const str = self.readString(ptr, len);
          console.info('[INFO]', str);
          return 0;
        },

        _log_warn(ptr, len) {
          const str = self.readString(ptr, len);
          console.warn('[WARN]', str);
          return 0;
        },

        _log_error(ptr, len) {
          const str = self.readString(ptr, len);
          console.error('[ERROR]', str);
          return 0;
        },

        _log_debug(ptr, len) {
          const str = self.readString(ptr, len);
          console.debug('[DEBUG]', str);
          return 0;
        },

        // ================================================================
        // TYPE CONVERSION
        // ================================================================

        _int_to_string(value, outPtr) {
          const str = value.toString();
          return self.writeString(str, outPtr);
        },

        _float_to_string(value, outPtr) {
          const str = value.toString();
          return self.writeString(str, outPtr);
        },

        _parse_int(ptr, len) {
          const str = self.readString(ptr, len);
          return parseInt(str, 10) || 0;
        },

        _parse_float(ptr, len) {
          const str = self.readString(ptr, len);
          return parseFloat(str) || 0.0;
        },

        // ================================================================
        // MEMORY
        // ================================================================

        _memory_copy(dest, src, len) {
          const view = new Uint8Array(self.memory.buffer);
          view.copyWithin(dest, src, src + len);
          return 0;
        },

        _memory_fill(ptr, value, len) {
          const view = new Uint8Array(self.memory.buffer);
          view.fill(value, ptr, ptr + len);
          return 0;
        },

        // ================================================================
        // ASSERTIONS (for debugging)
        // ================================================================

        _assert(condition, msgPtr, msgLen) {
          if (!condition) {
            const msg = self.readString(msgPtr, msgLen);
            console.error('[ASSERT FAILED]', msg);
            throw new Error(`Assertion failed: ${msg}`);
          }
          return 0;
        },

        _panic(msgPtr, msgLen) {
          const msg = self.readString(msgPtr, msgLen);
          console.error('[PANIC]', msg);
          throw new Error(`Panic: ${msg}`);
        }
      };
    }

    /**
     * Read string from WASM memory
     */
    readString(ptr, len) {
      if (!this.memory) return '';
      const bytes = new Uint8Array(this.memory.buffer, ptr, len);
      return new TextDecoder().decode(bytes);
    }

    /**
     * Write string to WASM memory
     */
    writeString(str, ptr) {
      if (!this.memory) return 0;
      const bytes = new TextEncoder().encode(str);
      const view = new Uint8Array(this.memory.buffer, ptr, bytes.length);
      view.set(bytes);
      return bytes.length;
    }

    /**
     * Clean up resources
     */
    destroy() {
      this.bridge.destroy();
      this.instance = null;
      this.memory = null;
    }
  }

  /**
   * Clean Canvas Application instance
   */
  class CleanCanvasApp {
    constructor(loader) {
      this.loader = loader;
      this.instance = loader.instance;
      this.exports = loader.instance.exports;
      this.paused = false;
    }

    /**
     * Call the start() function
     */
    start() {
      if (this.exports.start) {
        console.log('[CleanCanvas] Calling start()...');
        this.exports.start();
      } else if (this.exports._start) {
        console.log('[CleanCanvas] Calling _start()...');
        this.exports._start();
      } else if (this.exports.main) {
        console.log('[CleanCanvas] Calling main()...');
        this.exports.main();
      } else {
        console.warn('[CleanCanvas] No start(), _start(), or main() function found');
      }
      return this;
    }

    /**
     * Pause the animation loop
     */
    pause() {
      if (!this.paused) {
        this.paused = true;
        const bridge = this.loader.bridge;
        for (const [frameId, rafId] of bridge.animationFrames) {
          cancelAnimationFrame(rafId);
        }
        console.log('[CleanCanvas] Paused');
      }
      return this;
    }

    /**
     * Resume the animation loop
     */
    resume() {
      if (this.paused) {
        this.paused = false;
        const bridge = this.loader.bridge;
        const imports = bridge.getImports(this.loader.memory);

        // Re-request animation frames for all canvases
        for (const canvasId of bridge.canvases.keys()) {
          imports.env._canvas_request_frame(canvasId);
        }
        console.log('[CleanCanvas] Resumed');
      }
      return this;
    }

    /**
     * Toggle pause/resume
     */
    togglePause() {
      return this.paused ? this.resume() : this.pause();
    }

    /**
     * Get current FPS
     */
    getFPS() {
      return this.loader.bridge.fps;
    }

    /**
     * Get delta time
     */
    getDeltaTime() {
      return this.loader.bridge.deltaTime;
    }

    /**
     * Get elapsed time since start
     */
    getTime() {
      return (performance.now() - this.loader.bridge.startTime) / 1000;
    }

    /**
     * Call a named export function
     */
    call(name, ...args) {
      if (this.exports[name]) {
        return this.exports[name](...args);
      }
      console.warn(`[CleanCanvas] Function not found: ${name}`);
      return undefined;
    }

    /**
     * Check if function exists
     */
    hasFunction(name) {
      return typeof this.exports[name] === 'function';
    }

    /**
     * Get canvas bridge
     */
    get canvas() {
      return this.loader.bridge;
    }

    /**
     * Get canvas element by ID
     */
    getCanvasElement(id = 1) {
      return this.loader.bridge.getCanvasElement(id);
    }

    /**
     * Set master volume
     */
    setVolume(volume) {
      this.loader.bridge.masterVolume = volume;
      if (this.loader.bridge.currentMusicElement) {
        this.loader.bridge.currentMusicElement.volume =
          volume * this.loader.bridge.musicVolume *
          (this.loader.bridge.isMuted ? 0 : 1);
      }
      return this;
    }

    /**
     * Mute all audio
     */
    mute() {
      this.loader.bridge.isMuted = true;
      if (this.loader.bridge.currentMusicElement) {
        this.loader.bridge.currentMusicElement.volume = 0;
      }
      return this;
    }

    /**
     * Unmute all audio
     */
    unmute() {
      this.loader.bridge.isMuted = false;
      if (this.loader.bridge.currentMusicElement) {
        this.loader.bridge.currentMusicElement.volume =
          this.loader.bridge.musicVolume * this.loader.bridge.masterVolume;
      }
      return this;
    }

    /**
     * Toggle mute
     */
    toggleMute() {
      return this.loader.bridge.isMuted ? this.unmute() : this.mute();
    }

    /**
     * Destroy the application
     */
    destroy() {
      this.loader.destroy();
    }
  }

  // ============================================================================
  // GLOBAL API
  // ============================================================================

  const CleanCanvas = {
    /**
     * Load a WASM module
     * @param {string} url - URL to the .wasm file
     * @param {Object} options - Load options
     * @returns {Promise<CleanCanvasApp>} Application instance
     */
    async load(url, options = {}) {
      const loader = new CleanCanvasLoader();
      const app = await loader.load(url, options);

      // Auto-start unless disabled
      if (options.autoStart !== false) {
        app.start();
      }

      return app;
    },

    /**
     * Create a new loader instance (for advanced use)
     */
    createLoader() {
      return new CleanCanvasLoader();
    },

    /**
     * Get version
     */
    version: '2.0.0'
  };

  // Export globally
  if (typeof window !== 'undefined') {
    window.CleanCanvas = CleanCanvas;
    window.CleanCanvasLoader = CleanCanvasLoader;
    window.CleanCanvasApp = CleanCanvasApp;
  }

  // Export for module systems
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { CleanCanvas, CleanCanvasLoader, CleanCanvasApp };
  }

})();
