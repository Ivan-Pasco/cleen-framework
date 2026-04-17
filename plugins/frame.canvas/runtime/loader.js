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
  const WASM_PAGE_SIZE = 65536;

  // Standard tier fallback per platform-architecture/MEMORY_POLICY.md §3
  const CANVAS_TIER_INITIAL_PAGES = 256;
  const CANVAS_TIER_MAX_PAGES = 1024;

  // --- WASM binary parsing (reads module's declared memory) ---
  // See platform-architecture/MEMORY_POLICY.md §6.2

  function _readULEB128(bytes, ref) {
    let result = 0, shift = 0, byte;
    do {
      if (ref.offset >= bytes.length) throw new Error('ULEB128 truncated');
      byte = bytes[ref.offset++];
      result |= (byte & 0x7f) << shift;
      shift += 7;
      if (shift > 35) throw new Error('ULEB128 overflow');
    } while (byte & 0x80);
    return result >>> 0;
  }

  function _readWasmString(bytes, ref) {
    const len = _readULEB128(bytes, ref);
    const str = new TextDecoder().decode(bytes.subarray(ref.offset, ref.offset + len));
    ref.offset += len;
    return str;
  }

  function _readMemoryLimits(bytes, ref) {
    const flags = _readULEB128(bytes, ref);
    const initial = _readULEB128(bytes, ref);
    const max = (flags & 0x01) ? _readULEB128(bytes, ref) : null;
    return { initial, max };
  }

  function parseWasmMemoryDecl(wasmBuffer) {
    const bytes = new Uint8Array(wasmBuffer);
    if (bytes.length < 8) return null;
    if (bytes[0] !== 0x00 || bytes[1] !== 0x61 || bytes[2] !== 0x73 || bytes[3] !== 0x6d) return null;

    const ref = { offset: 8 };
    let fromCustom = null, fromImport = null, fromMemory = null;

    while (ref.offset < bytes.length) {
      const sectionId = bytes[ref.offset++];
      const sectionSize = _readULEB128(bytes, ref);
      const sectionEnd = ref.offset + sectionSize;

      try {
        if (sectionId === 0) {
          const name = _readWasmString(bytes, ref);
          if (name === 'clean:memory') {
            // Format: initial_pages:ULEB128, max_pages:ULEB128 (tier string may follow; ignored here)
            const initial = _readULEB128(bytes, ref);
            const max = _readULEB128(bytes, ref);
            fromCustom = { initial, max, source: 'clean:memory' };
          }
        } else if (sectionId === 2) {
          const count = _readULEB128(bytes, ref);
          for (let i = 0; i < count; i++) {
            _readWasmString(bytes, ref); // module
            _readWasmString(bytes, ref); // name
            const kind = bytes[ref.offset++];
            if (kind === 0) {
              _readULEB128(bytes, ref); // typeidx
            } else if (kind === 1) {
              ref.offset++; // reftype
              _readMemoryLimits(bytes, ref);
            } else if (kind === 2) {
              const limits = _readMemoryLimits(bytes, ref);
              if (!fromImport) fromImport = { ...limits, source: 'import' };
            } else if (kind === 3) {
              ref.offset += 2; // valtype + mut
            }
          }
        } else if (sectionId === 5) {
          const count = _readULEB128(bytes, ref);
          if (count > 0 && !fromMemory) {
            fromMemory = { ..._readMemoryLimits(bytes, ref), source: 'memory-section' };
          }
        }
      } catch (_) {
        // tolerate malformed section — fall through to next
      }

      ref.offset = sectionEnd;
    }

    return fromCustom || fromImport || fromMemory;
  }

  // --- window.__cleanRuntime registration (MEMORY_POLICY.md §9.3) ---

  function registerCleanRuntime(statsProvider) {
    if (typeof window === 'undefined') return;
    if (!window.__cleanRuntime) {
      window.__cleanRuntime = {
        _providers: [],
        memoryStats() {
          const p = this._providers[this._providers.length - 1];
          return p ? p() : null;
        }
      };
    }
    window.__cleanRuntime._providers.push(statsProvider);
  }

  /**
   * Clean Canvas Application Loader
   */
  class CleanCanvasLoader {
    constructor() {
      this.bridge = new CanvasBridge();
      this.memory = null;
      this.instance = null;
      this.stringAllocOffset = STRING_BUFFER_OFFSET;
      this._initialPages = 0;
      this._maxPages = 0;
      this._growCount = 0;
      this._lastBufferSize = 0;
      this._heapPtr = 0;
    }

    _checkMemoryGrowth() {
      if (!this.memory) return;
      const size = this.memory.buffer.byteLength;
      if (size !== this._lastBufferSize) {
        const fromPages = this._lastBufferSize / WASM_PAGE_SIZE;
        const toPages = size / WASM_PAGE_SIZE;
        const fromMB = (this._lastBufferSize / (1024 * 1024)).toFixed(1);
        const toMB = (size / (1024 * 1024)).toFixed(1);
        console.warn(`[clean] Memory grew: ${fromPages} → ${toPages} pages (${fromMB} MB → ${toMB} MB)`);
        this._growCount++;
        this._lastBufferSize = size;
      }
    }

    _memoryStats() {
      const size = this.memory ? this.memory.buffer.byteLength : 0;
      return {
        currentPages: size / WASM_PAGE_SIZE,
        currentBytes: size,
        maxPages: this._maxPages,
        growCount: this._growCount,
        heapPtr: this._heapPtr,
      };
    }

    /**
     * Load a WASM module
     * @param {string} url - URL to the .wasm file
     * @param {Object} options - Load options
     * @returns {Promise<CleanCanvasApp>} Application instance
     */
    async load(url, options = {}) {
      console.log(`[CleanCanvas] Loading ${url}...`);

      // Fetch WASM first so we can inspect its memory declaration
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to fetch ${url}: ${response.status}`);
      }
      const wasmBuffer = await response.arrayBuffer();

      // Resolve initial/max pages per MEMORY_POLICY.md §6.2:
      //   1. Caller override (options.memoryPages)
      //   2. Module declaration (clean:memory custom section, then import descriptor, then memory section)
      //   3. Canvas tier defaults (256 / 1024)
      const decl = parseWasmMemoryDecl(wasmBuffer);
      let initialPages, maxPages;
      if (options.memoryPages) {
        initialPages = options.memoryPages;
        maxPages = options.memoryMaxPages || options.memoryPages * 4;
      } else if (decl) {
        initialPages = decl.initial;
        maxPages = decl.max != null ? decl.max : CANVAS_TIER_MAX_PAGES;
        console.log(`[CleanCanvas] Memory from WASM ${decl.source}: initial=${initialPages} max=${maxPages}`);
      } else {
        initialPages = CANVAS_TIER_INITIAL_PAGES;
        maxPages = CANVAS_TIER_MAX_PAGES;
        console.log(`[CleanCanvas] No memory declaration found; using canvas tier defaults (${initialPages}/${maxPages})`);
      }

      this._initialPages = initialPages;
      this._maxPages = maxPages;
      this.memory = new WebAssembly.Memory({ initial: initialPages, maximum: maxPages });
      this._lastBufferSize = this.memory.buffer.byteLength;

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
        const { instance } = await WebAssembly.instantiate(wasmBuffer, imports);

        this.instance = instance;
        this.bridge.setWasmInstance(instance);

        if (instance.exports.__heap_ptr && typeof instance.exports.__heap_ptr.value === 'number') {
          this._heapPtr = instance.exports.__heap_ptr.value;
        }

        // Check for module-initiated growth during instantiation/start
        this._checkMemoryGrowth();

        registerCleanRuntime(() => this._memoryStats());

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
      this._checkMemoryGrowth();
      const bytes = new Uint8Array(this.memory.buffer, ptr, len);
      return new TextDecoder().decode(bytes);
    }

    /**
     * Write string to WASM memory
     */
    writeString(str, ptr) {
      if (!this.memory) return 0;
      this._checkMemoryGrowth();
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
