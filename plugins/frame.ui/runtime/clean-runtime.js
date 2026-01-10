/**
 * Clean UI Client Runtime
 * Full-featured runtime for Clean UI islands architecture, SPAs, and games
 *
 * Features:
 * - Finds [data-screen] elements and hydrates them
 * - Parses hydration JSON for state and events
 * - Creates reactive state proxy with automatic re-renders
 * - Full event handler support:
 *   - click, dblclick, change, input, submit
 *   - focus, blur
 *   - keydown, keyup, keypress
 *   - mouseenter, mouseleave, mousedown, mouseup, mousemove
 * - Rich event objects with keyboard/mouse properties
 * - Two-way binding for form inputs (text, checkbox, radio, select, range)
 * - Hydration modes: on, visible, idle, only (client-only rendering)
 * - Client-side navigation (nav.go, nav.back, nav.replace)
 * - App-level state management (shared across screens)
 * - Screen lifecycle hooks (onMount, onUnmount, onUpdate)
 * - History API integration with popstate handling
 * - Lazy-loading support for client-only screens
 * - Canvas 2D graphics:
 *   - Drawing primitives: rect, circle, ellipse, line, triangle, text, image
 *   - Path operations: beginPath, moveTo, lineTo, closePath, fill, stroke
 *   - Transformations: save, restore, translate, rotate, scale
 *   - Animation loop with requestAnimationFrame (60fps)
 *   - Mouse and touch input handling
 *   - Keyboard input (when canvas focused)
 *   - Sprite sheet support for game graphics
 * - Theming system:
 *   - CSS variables for colors, spacing, typography
 *   - Theme registration and switching
 *   - Dark mode toggle
 * - Form validation:
 *   - Built-in validators: required, email, minLength, maxLength, pattern, url
 *   - Custom validator registration
 *   - Real-time validation feedback
 *   - Form submission prevention on invalid
 * - Accessibility:
 *   - Keyboard navigation for lists/menus
 *   - Focus trapping for modals
 *   - Screen reader announcements
 *   - Skip links for keyboard users
 *   - Color contrast checking (WCAG AA/AAA)
 *
 * Size target: ~20KB minified
 */

(function() {
  'use strict';

  // Store for screen instances
  const screens = new Map();

  /**
   * Initialize all screens on the page
   */
  function init() {
    // Initialize navigation system
    initNavigation();

    // Find all screen containers
    const screenElements = document.querySelectorAll('[data-screen]');

    screenElements.forEach(element => {
      const screenName = element.getAttribute('data-screen');
      const clientMode = element.getAttribute('data-client') || 'off';

      if (clientMode === 'off') {
        return; // SSR only, no hydration needed
      }

      // Handle client-only mode (no SSR, render entirely on client)
      if (clientMode === 'only') {
        renderClientOnly(element, screenName);
        return;
      }

      // Find hydration data
      const hydrationScript = document.querySelector(
        `script[data-hydration="${screenName}"]`
      );

      if (!hydrationScript) {
        console.warn(`[Clean Runtime] No hydration data found for screen: ${screenName}`);
        return;
      }

      try {
        const hydrationData = JSON.parse(hydrationScript.textContent);

        // Hydrate based on client mode
        if (clientMode === 'on') {
          hydrateScreen(element, screenName, hydrationData);
        } else if (clientMode === 'visible') {
          hydrateOnVisible(element, screenName, hydrationData);
        } else if (clientMode === 'idle') {
          hydrateOnIdle(element, screenName, hydrationData);
        }
      } catch (e) {
        console.error(`[Clean Runtime] Failed to parse hydration data for ${screenName}:`, e);
      }
    });
  }

  /**
   * Render a screen entirely on the client (no SSR)
   */
  function renderClientOnly(element, screenName) {
    // Check if screen definition is registered
    const screenDef = window.__cleanScreens && window.__cleanScreens[screenName];

    if (!screenDef) {
      console.warn(`[Clean Runtime] Screen definition not found for client-only: ${screenName}`);
      element.innerHTML = `<div class="ui-loading">Loading ${screenName}...</div>`;

      // Try to lazy-load the screen definition
      loadScreenDefinition(screenName).then(def => {
        if (def) {
          renderClientOnlyWithDef(element, screenName, def);
        } else {
          element.innerHTML = `<div class="ui-error">Failed to load ${screenName}</div>`;
        }
      });
      return;
    }

    renderClientOnlyWithDef(element, screenName, screenDef);
  }

  /**
   * Render client-only screen with definition
   */
  function renderClientOnlyWithDef(element, screenName, screenDef) {
    const state = screenDef.initialState ? { ...screenDef.initialState } : {};
    const events = screenDef.events || [];

    // Create reactive state
    const reactiveState = createReactiveState(state, () => {
      // Re-render on state change
      if (screenDef.render) {
        element.innerHTML = screenDef.render(reactiveState);
        attachEvents(element, events, reactiveState);
      }
    });

    // Store screen instance
    screens.set(screenName, {
      element,
      state: reactiveState,
      hydrationData: { state, events },
      onMount: screenDef.onMount,
      onUnmount: screenDef.onUnmount,
      onUpdate: screenDef.onUpdate
    });

    // Initial render
    if (screenDef.render) {
      element.innerHTML = screenDef.render(reactiveState);
      attachEvents(element, events, reactiveState);
    }

    // Mark as hydrated
    element.setAttribute('data-hydrated', 'true');

    // Call onMount
    if (screenDef.onMount) {
      screenDef.onMount(navState.params);
    }

    console.log(`[Clean Runtime] Client-only rendered: ${screenName}`);
  }

  /**
   * Lazy-load a screen definition
   */
  async function loadScreenDefinition(screenName) {
    try {
      // Try to load from a conventional path
      const response = await fetch(`/screens/${screenName}.js`);
      if (response.ok) {
        const script = document.createElement('script');
        script.textContent = await response.text();
        document.head.appendChild(script);

        // Return the registered definition
        return window.__cleanScreens && window.__cleanScreens[screenName];
      }
    } catch (e) {
      console.error(`[Clean Runtime] Failed to load screen: ${screenName}`, e);
    }
    return null;
  }

  /**
   * Hydrate a screen immediately
   */
  function hydrateScreen(element, screenName, hydrationData) {
    const state = hydrationData.state || {};
    const events = hydrationData.events || [];
    const lifecycle = hydrationData.lifecycle || {};

    // Create lifecycle hook functions
    const createHookFn = (actionCode) => {
      if (!actionCode) return null;
      return (params) => {
        const context = { ...reactiveState, nav: window.nav, app: appState || {}, params };
        try {
          const fn = new Function(...Object.keys(context), actionCode);
          fn(...Object.values(context));
        } catch (e) {
          console.error(`[Clean Runtime] Lifecycle hook error:`, e);
        }
      };
    };

    // Create reactive state
    const reactiveState = createReactiveState(state, () => {
      rerender(element, screenName, reactiveState);
      // Call onUpdate hook
      const screen = screens.get(screenName);
      if (screen && screen.onUpdate) {
        screen.onUpdate();
      }
    });

    // Store screen instance with lifecycle hooks
    screens.set(screenName, {
      element,
      state: reactiveState,
      hydrationData,
      onMount: createHookFn(lifecycle.onMount),
      onUnmount: createHookFn(lifecycle.onUnmount),
      onUpdate: createHookFn(lifecycle.onUpdate)
    });

    // Attach event listeners
    attachEvents(element, events, reactiveState);

    // Mark as hydrated
    element.setAttribute('data-hydrated', 'true');
    element.setAttribute('data-active', 'true');

    // Initialize any canvases in this screen
    initCanvases(element);

    // Call onMount hook
    const screen = screens.get(screenName);
    if (screen && screen.onMount) {
      screen.onMount(navState.params);
    }

    console.log(`[Clean Runtime] Hydrated screen: ${screenName}`);
  }

  /**
   * Hydrate when element becomes visible (IntersectionObserver)
   */
  function hydrateOnVisible(element, screenName, hydrationData) {
    if ('IntersectionObserver' in window) {
      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            observer.disconnect();
            hydrateScreen(element, screenName, hydrationData);
          }
        });
      }, { threshold: 0.1 });

      observer.observe(element);
    } else {
      // Fallback to immediate hydration
      hydrateScreen(element, screenName, hydrationData);
    }
  }

  /**
   * Hydrate when browser is idle (requestIdleCallback)
   */
  function hydrateOnIdle(element, screenName, hydrationData) {
    if ('requestIdleCallback' in window) {
      requestIdleCallback(() => {
        hydrateScreen(element, screenName, hydrationData);
      });
    } else {
      // Fallback to setTimeout
      setTimeout(() => {
        hydrateScreen(element, screenName, hydrationData);
      }, 200);
    }
  }

  /**
   * Create a reactive state proxy
   */
  function createReactiveState(initialState, onChange) {
    return new Proxy({ ...initialState }, {
      set(target, property, value) {
        const oldValue = target[property];
        target[property] = value;

        if (oldValue !== value) {
          // Debounce re-renders
          if (target._renderTimeout) {
            clearTimeout(target._renderTimeout);
          }
          target._renderTimeout = setTimeout(() => {
            onChange();
          }, 0);
        }

        return true;
      },
      get(target, property) {
        return target[property];
      }
    });
  }

  /**
   * Attach event listeners to elements
   */
  function attachEvents(container, events, state) {
    events.forEach(eventDef => {
      const { id, type, action } = eventDef;
      const element = container.querySelector(`[data-event-id="${id}"]`);

      if (!element) {
        console.warn(`[Clean Runtime] Event target not found: ${id}`);
        return;
      }

      // Map event type
      const eventType = mapEventType(type);

      element.addEventListener(eventType, (domEvent) => {
        const cleanEvent = createCleanEvent(domEvent, element);
        executeAction(action, state, cleanEvent);
      });
    });

    // Also attach listeners to bound elements for two-way binding
    const boundElements = container.querySelectorAll('[data-bind]');
    boundElements.forEach(element => {
      const bindVar = element.getAttribute('data-bind');

      // Handle different input types
      if (element.type === 'checkbox') {
        element.addEventListener('change', () => {
          state[bindVar] = element.checked;
        });
      } else if (element.type === 'radio') {
        element.addEventListener('change', () => {
          if (element.checked) {
            state[bindVar] = element.value;
          }
        });
      } else if (element.tagName === 'SELECT') {
        element.addEventListener('change', () => {
          state[bindVar] = element.value;
        });
      } else {
        // Text input, textarea, range
        element.addEventListener('input', () => {
          if (element.type === 'range' || element.type === 'number') {
            state[bindVar] = parseFloat(element.value) || 0;
          } else {
            state[bindVar] = element.value;
          }
        });
      }
    });
  }

  /**
   * Map event type names
   */
  function mapEventType(type) {
    const mapping = {
      'click': 'click',
      'onClick': 'click',
      'dblclick': 'dblclick',
      'onDoubleClick': 'dblclick',
      'change': 'change',
      'onChange': 'change',
      'input': 'input',
      'onInput': 'input',
      'submit': 'submit',
      'onSubmit': 'submit',
      'focus': 'focus',
      'onFocus': 'focus',
      'blur': 'blur',
      'onBlur': 'blur',
      'keydown': 'keydown',
      'onKeyDown': 'keydown',
      'keyup': 'keyup',
      'onKeyUp': 'keyup',
      'keypress': 'keypress',
      'onKeyPress': 'keypress',
      'mouseenter': 'mouseenter',
      'onMouseEnter': 'mouseenter',
      'mouseleave': 'mouseleave',
      'onMouseLeave': 'mouseleave',
      'mousedown': 'mousedown',
      'onMouseDown': 'mousedown',
      'mouseup': 'mouseup',
      'onMouseUp': 'mouseup',
      'mousemove': 'mousemove',
      'onMouseMove': 'mousemove'
    };
    return mapping[type] || type;
  }

  /**
   * Create a Clean-friendly event object from a DOM event
   */
  function createCleanEvent(domEvent, element) {
    const cleanEvent = {
      type: domEvent.type,
      target: element,
      preventDefault: () => domEvent.preventDefault(),
      stopPropagation: () => domEvent.stopPropagation()
    };

    // Input-specific properties
    if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA' || element.tagName === 'SELECT') {
      cleanEvent.value = element.value;

      if (element.type === 'checkbox' || element.type === 'radio') {
        cleanEvent.checked = element.checked;
      }
    }

    // Keyboard event properties
    if (domEvent instanceof KeyboardEvent) {
      cleanEvent.key = domEvent.key;
      cleanEvent.code = domEvent.code;
      cleanEvent.ctrlKey = domEvent.ctrlKey;
      cleanEvent.shiftKey = domEvent.shiftKey;
      cleanEvent.altKey = domEvent.altKey;
      cleanEvent.metaKey = domEvent.metaKey;
      cleanEvent.repeat = domEvent.repeat;
    }

    // Mouse event properties
    if (domEvent instanceof MouseEvent) {
      cleanEvent.x = domEvent.clientX;
      cleanEvent.y = domEvent.clientY;
      cleanEvent.clientX = domEvent.clientX;
      cleanEvent.clientY = domEvent.clientY;
      cleanEvent.pageX = domEvent.pageX;
      cleanEvent.pageY = domEvent.pageY;
      cleanEvent.offsetX = domEvent.offsetX;
      cleanEvent.offsetY = domEvent.offsetY;
      cleanEvent.button = domEvent.button;
      cleanEvent.buttons = domEvent.buttons;
      cleanEvent.ctrlKey = domEvent.ctrlKey;
      cleanEvent.shiftKey = domEvent.shiftKey;
      cleanEvent.altKey = domEvent.altKey;
    }

    return cleanEvent;
  }

  /**
   * Execute an action string with state context
   */
  function executeAction(action, state, event) {
    try {
      // Create a safe execution context
      const context = {
        ...state,
        event,
        // Navigation
        nav: window.nav,
        // App state
        app: appState || {},
        // Helper functions
        parseInt: parseInt,
        parseFloat: parseFloat,
        Math: Math,
        String: String,
        Number: Number,
        Boolean: Boolean,
        Array: Array,
        JSON: JSON,
        console: console
      };

      // Parse and execute the action
      // Simple assignments: "count = count + 1"
      const assignmentMatch = action.match(/^(\w+)\s*=\s*(.+)$/);
      if (assignmentMatch) {
        const [, varName, expression] = assignmentMatch;

        // Evaluate the expression
        const evalFunc = new Function(
          ...Object.keys(context),
          `return ${expression}`
        );
        const result = evalFunc(...Object.values(context));

        // Update state
        state[varName] = result;
        return;
      }

      // Function calls or other expressions
      const evalFunc = new Function(
        ...Object.keys(context),
        action
      );
      evalFunc(...Object.values(context));

    } catch (e) {
      console.error(`[Clean Runtime] Error executing action: ${action}`, e);
    }
  }

  /**
   * Re-render a screen (placeholder - actual implementation depends on screen class)
   */
  function rerender(element, screenName, state) {
    // For now, trigger a custom event that the server-rendered code can listen to
    const event = new CustomEvent('clean:rerender', {
      detail: { screenName, state: { ...state } }
    });
    element.dispatchEvent(event);

    // Simple client-side re-render for basic widgets
    // Update text elements that display state values
    Object.keys(state).forEach(key => {
      if (key.startsWith('_')) return; // Skip internal properties

      const value = state[key];

      // Find elements with data-bind attribute
      const boundElements = element.querySelectorAll(`[data-bind="${key}"]`);
      boundElements.forEach(el => {
        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {
          if (el.type === 'checkbox') {
            el.checked = !!value;
          } else if (el.type === 'radio') {
            el.checked = (el.value === String(value));
          } else {
            el.value = value;
          }
        } else if (el.tagName === 'SELECT') {
          el.value = value;
        } else {
          el.textContent = value;
        }
      });

      // Find text elements that might display this value
      // This is a simple heuristic - looks for spans containing the value
      const textElements = element.querySelectorAll('.ui-text');
      textElements.forEach(el => {
        // Update if the element's data attribute matches
        if (el.getAttribute('data-state-key') === key) {
          el.textContent = value;
        }
      });
    });

    console.log(`[Clean Runtime] Re-rendered screen: ${screenName}`, state);
  }

  // ============================================================================
  // CANVAS SYSTEM
  // ============================================================================

  /**
   * Store for canvas instances
   */
  const canvases = new Map();

  /**
   * Sprite sheet cache
   */
  const spriteCache = new Map();

  /**
   * Initialize all canvas elements on the page
   */
  function initCanvases(container) {
    const canvasElements = container.querySelectorAll('canvas[data-canvas-id]');

    canvasElements.forEach(canvas => {
      const canvasId = canvas.getAttribute('data-canvas-id');
      const animate = canvas.getAttribute('data-animate') === 'true';
      const drawCode = canvas.getAttribute('data-draw') || '';
      const updateCode = canvas.getAttribute('data-update') || '';
      const eventsAttr = canvas.getAttribute('data-canvas-events') || '';

      initCanvas(canvas, canvasId, animate, drawCode, updateCode, eventsAttr);
    });
  }

  /**
   * Initialize a single canvas
   */
  function initCanvas(canvas, canvasId, animate, drawCode, updateCode, eventsAttr) {
    const ctx = canvas.getContext('2d');

    // Create canvas API object
    const canvasAPI = createCanvasAPI(ctx, canvas);

    // Parse event handlers
    const eventHandlers = {};
    if (eventsAttr) {
      eventsAttr.split(',').forEach(pair => {
        const [type, action] = pair.split(':');
        if (type && action) {
          eventHandlers[type.trim()] = action.trim();
        }
      });
    }

    // Store canvas instance
    const instance = {
      canvas,
      ctx,
      api: canvasAPI,
      drawCode,
      updateCode,
      animate,
      running: false,
      frameCount: 0,
      startTime: performance.now(),
      lastFrameTime: performance.now(),
      mouseX: 0,
      mouseY: 0,
      mouseDown: false,
      touches: [],
      keysDown: new Set()
    };

    canvases.set(canvasId, instance);

    // Attach input handlers
    attachCanvasInputHandlers(canvas, instance, eventHandlers);

    // Execute initial draw
    if (drawCode) {
      executeCanvasDraw(instance, drawCode);
    }

    // Start animation loop if enabled
    if (animate) {
      startAnimationLoop(instance, updateCode, drawCode);
    }

    console.log(`[Clean Runtime] Canvas initialized: ${canvasId}`);
  }

  /**
   * Create the canvas drawing API
   */
  function createCanvasAPI(ctx, canvas) {
    return {
      // Properties
      get width() { return canvas.width; },
      get height() { return canvas.height; },

      // Clear
      clear(color) {
        if (color) {
          ctx.fillStyle = color;
          ctx.fillRect(0, 0, canvas.width, canvas.height);
        } else {
          ctx.clearRect(0, 0, canvas.width, canvas.height);
        }
      },

      // Rectangle
      rect(x, y, width, height, options = {}) {
        ctx.beginPath();
        ctx.rect(x, y, width, height);
        if (options.fill) {
          ctx.fillStyle = options.fill;
          ctx.fill();
        }
        if (options.stroke) {
          ctx.strokeStyle = options.stroke;
          ctx.lineWidth = options.lineWidth || 1;
          ctx.stroke();
        }
      },

      // Circle
      circle(x, y, radius, options = {}) {
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, Math.PI * 2);
        if (options.fill) {
          ctx.fillStyle = options.fill;
          ctx.fill();
        }
        if (options.stroke) {
          ctx.strokeStyle = options.stroke;
          ctx.lineWidth = options.lineWidth || 1;
          ctx.stroke();
        }
      },

      // Ellipse
      ellipse(x, y, radiusX, radiusY, options = {}) {
        ctx.beginPath();
        ctx.ellipse(x, y, radiusX, radiusY, 0, 0, Math.PI * 2);
        if (options.fill) {
          ctx.fillStyle = options.fill;
          ctx.fill();
        }
        if (options.stroke) {
          ctx.strokeStyle = options.stroke;
          ctx.lineWidth = options.lineWidth || 1;
          ctx.stroke();
        }
      },

      // Line
      line(x1, y1, x2, y2, options = {}) {
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.strokeStyle = options.stroke || '#000';
        ctx.lineWidth = options.lineWidth || 1;
        ctx.stroke();
      },

      // Triangle
      triangle(x1, y1, x2, y2, x3, y3, options = {}) {
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.lineTo(x3, y3);
        ctx.closePath();
        if (options.fill) {
          ctx.fillStyle = options.fill;
          ctx.fill();
        }
        if (options.stroke) {
          ctx.strokeStyle = options.stroke;
          ctx.lineWidth = options.lineWidth || 1;
          ctx.stroke();
        }
      },

      // Path operations
      beginPath() { ctx.beginPath(); },
      moveTo(x, y) { ctx.moveTo(x, y); },
      lineTo(x, y) { ctx.lineTo(x, y); },
      closePath() { ctx.closePath(); },
      fill(color) {
        if (color) ctx.fillStyle = color;
        ctx.fill();
      },
      stroke(color, lineWidth) {
        if (color) ctx.strokeStyle = color;
        if (lineWidth) ctx.lineWidth = lineWidth;
        ctx.stroke();
      },

      // Text
      text(str, x, y, options = {}) {
        ctx.font = options.font || '16px sans-serif';
        if (options.fill) {
          ctx.fillStyle = options.fill;
          ctx.fillText(str, x, y);
        } else {
          ctx.fillStyle = '#000';
          ctx.fillText(str, x, y);
        }
        if (options.stroke) {
          ctx.strokeStyle = options.stroke;
          ctx.lineWidth = options.lineWidth || 1;
          ctx.strokeText(str, x, y);
        }
      },

      // Image
      image(src, x, y, width, height) {
        let img = spriteCache.get(src);
        if (!img) {
          img = new Image();
          img.src = src;
          spriteCache.set(src, img);
        }
        if (img.complete) {
          if (width && height) {
            ctx.drawImage(img, x, y, width, height);
          } else {
            ctx.drawImage(img, x, y);
          }
        }
      },

      // Sprite from spritesheet
      sprite(src, sx, sy, sw, sh, dx, dy, dw, dh) {
        let img = spriteCache.get(src);
        if (!img) {
          img = new Image();
          img.src = src;
          spriteCache.set(src, img);
        }
        if (img.complete) {
          ctx.drawImage(img, sx, sy, sw, sh, dx, dy, dw || sw, dh || sh);
        }
      },

      // Transformations
      save() { ctx.save(); },
      restore() { ctx.restore(); },
      translate(x, y) { ctx.translate(x, y); },
      rotate(angle) { ctx.rotate(angle * Math.PI / 180); },
      scale(x, y) { ctx.scale(x, y); },

      // Global alpha
      setAlpha(alpha) { ctx.globalAlpha = alpha; },

      // Composite operation
      setComposite(mode) { ctx.globalCompositeOperation = mode; }
    };
  }

  /**
   * Attach input handlers to canvas
   */
  function attachCanvasInputHandlers(canvas, instance, eventHandlers) {
    // Mouse move
    canvas.addEventListener('mousemove', (e) => {
      const rect = canvas.getBoundingClientRect();
      instance.mouseX = e.clientX - rect.left;
      instance.mouseY = e.clientY - rect.top;

      if (eventHandlers.mousemove) {
        executeCanvasEvent(instance, eventHandlers.mousemove, {
          x: instance.mouseX,
          y: instance.mouseY
        });
      }
    });

    // Mouse down
    canvas.addEventListener('mousedown', (e) => {
      instance.mouseDown = true;
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (eventHandlers.mousedown) {
        executeCanvasEvent(instance, eventHandlers.mousedown, { x, y, button: e.button });
      }
    });

    // Mouse up
    canvas.addEventListener('mouseup', (e) => {
      instance.mouseDown = false;
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (eventHandlers.mouseup) {
        executeCanvasEvent(instance, eventHandlers.mouseup, { x, y, button: e.button });
      }
    });

    // Click
    canvas.addEventListener('click', (e) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (eventHandlers.click) {
        executeCanvasEvent(instance, eventHandlers.click, { x, y });
      }
    });

    // Touch start
    canvas.addEventListener('touchstart', (e) => {
      e.preventDefault();
      instance.touches = Array.from(e.touches).map(t => {
        const rect = canvas.getBoundingClientRect();
        return { x: t.clientX - rect.left, y: t.clientY - rect.top };
      });

      if (eventHandlers.touchstart) {
        executeCanvasEvent(instance, eventHandlers.touchstart, {
          touches: instance.touches,
          x: instance.touches[0]?.x || 0,
          y: instance.touches[0]?.y || 0
        });
      }
    });

    // Touch move
    canvas.addEventListener('touchmove', (e) => {
      e.preventDefault();
      instance.touches = Array.from(e.touches).map(t => {
        const rect = canvas.getBoundingClientRect();
        return { x: t.clientX - rect.left, y: t.clientY - rect.top };
      });

      if (eventHandlers.touchmove) {
        executeCanvasEvent(instance, eventHandlers.touchmove, {
          touches: instance.touches,
          x: instance.touches[0]?.x || 0,
          y: instance.touches[0]?.y || 0
        });
      }
    });

    // Touch end
    canvas.addEventListener('touchend', (e) => {
      instance.touches = Array.from(e.touches).map(t => {
        const rect = canvas.getBoundingClientRect();
        return { x: t.clientX - rect.left, y: t.clientY - rect.top };
      });

      if (eventHandlers.touchend) {
        executeCanvasEvent(instance, eventHandlers.touchend, {
          touches: instance.touches
        });
      }
    });

    // Keyboard (when canvas is focused)
    canvas.setAttribute('tabindex', '0');
    canvas.addEventListener('keydown', (e) => {
      instance.keysDown.add(e.key);
    });
    canvas.addEventListener('keyup', (e) => {
      instance.keysDown.delete(e.key);
    });
  }

  /**
   * Execute canvas draw code
   */
  function executeCanvasDraw(instance, drawCode) {
    try {
      const canvas = instance.api;
      const fn = new Function('canvas', 'mouseX', 'mouseY', 'mouseDown', 'frameCount', 'time', 'deltaTime',
        drawCode.replace(/\\n/g, '\n'));
      fn(canvas, instance.mouseX, instance.mouseY, instance.mouseDown,
        instance.frameCount, (performance.now() - instance.startTime) / 1000,
        (performance.now() - instance.lastFrameTime) / 1000);
    } catch (e) {
      console.error('[Clean Runtime] Canvas draw error:', e);
    }
  }

  /**
   * Execute canvas event handler
   */
  function executeCanvasEvent(instance, action, event) {
    try {
      const canvas = instance.api;
      const fn = new Function('canvas', 'event', 'mouseX', 'mouseY',
        action.replace(/\\n/g, '\n'));
      fn(canvas, event, instance.mouseX, instance.mouseY);
    } catch (e) {
      console.error('[Clean Runtime] Canvas event error:', e);
    }
  }

  /**
   * Start animation loop for a canvas
   */
  function startAnimationLoop(instance, updateCode, drawCode) {
    if (instance.running) return;
    instance.running = true;

    function loop() {
      if (!instance.running) return;

      const now = performance.now();
      const deltaTime = (now - instance.lastFrameTime) / 1000;
      instance.lastFrameTime = now;
      instance.frameCount++;

      // Execute update
      if (updateCode) {
        try {
          const fn = new Function('deltaTime', 'time', 'frameCount', 'mouseX', 'mouseY', 'mouseDown', 'isKeyDown',
            updateCode.replace(/\\n/g, '\n'));
          fn(deltaTime, (now - instance.startTime) / 1000, instance.frameCount,
            instance.mouseX, instance.mouseY, instance.mouseDown,
            (key) => instance.keysDown.has(key));
        } catch (e) {
          console.error('[Clean Runtime] Canvas update error:', e);
        }
      }

      // Execute draw
      if (drawCode) {
        executeCanvasDraw(instance, drawCode);
      }

      requestAnimationFrame(loop);
    }

    requestAnimationFrame(loop);
  }

  /**
   * Stop animation loop
   */
  function stopAnimationLoop(canvasId) {
    const instance = canvases.get(canvasId);
    if (instance) {
      instance.running = false;
    }
  }

  /**
   * Pause animation
   */
  function pauseCanvas(canvasId) {
    stopAnimationLoop(canvasId);
  }

  /**
   * Resume animation
   */
  function resumeCanvas(canvasId) {
    const instance = canvases.get(canvasId);
    if (instance && instance.animate && !instance.running) {
      startAnimationLoop(instance, instance.updateCode, instance.drawCode);
    }
  }

  // ============================================================================
  // NAVIGATION SYSTEM
  // ============================================================================

  /**
   * Navigation state
   */
  const navState = {
    current: null,        // Current screen name
    params: {},           // Route parameters
    history: [],          // Navigation history stack
    listeners: []         // Navigation change listeners
  };

  /**
   * Initialize navigation from URL
   */
  function initNavigation() {
    // Parse initial route from URL
    const path = window.location.pathname;
    const hash = window.location.hash.slice(1);
    const search = new URLSearchParams(window.location.search);

    // Try to get screen name from hash or path
    let screenName = hash || path.split('/').pop() || 'Home';
    if (screenName === '' || screenName === 'index.html') {
      screenName = 'Home';
    }

    // Parse params from query string
    const params = {};
    search.forEach((value, key) => {
      // Try to parse as number or boolean
      if (value === 'true') params[key] = true;
      else if (value === 'false') params[key] = false;
      else if (!isNaN(value) && value !== '') params[key] = Number(value);
      else params[key] = value;
    });

    navState.current = screenName;
    navState.params = params;

    // Listen for browser back/forward
    window.addEventListener('popstate', handlePopState);
  }

  /**
   * Handle browser back/forward navigation
   */
  function handlePopState(event) {
    if (event.state) {
      const { screen, params } = event.state;
      navState.current = screen;
      navState.params = params || {};
      showScreen(screen, params, false);
    }
  }

  /**
   * Navigate to a screen
   */
  function navGo(screenName, params = {}) {
    // Save current to history
    navState.history.push({
      screen: navState.current,
      params: { ...navState.params }
    });

    // Update state
    navState.current = screenName;
    navState.params = params;

    // Update URL
    const url = buildUrl(screenName, params);
    window.history.pushState({ screen: screenName, params }, '', url);

    // Show the screen
    showScreen(screenName, params, true);

    // Notify listeners
    notifyNavListeners('go', screenName, params);
  }

  /**
   * Navigate back
   */
  function navBack() {
    if (navState.history.length > 0) {
      const prev = navState.history.pop();
      navState.current = prev.screen;
      navState.params = prev.params;

      const url = buildUrl(prev.screen, prev.params);
      window.history.pushState({ screen: prev.screen, params: prev.params }, '', url);

      showScreen(prev.screen, prev.params, true);
      notifyNavListeners('back', prev.screen, prev.params);
    } else {
      window.history.back();
    }
  }

  /**
   * Replace current screen (no history entry)
   */
  function navReplace(screenName, params = {}) {
    navState.current = screenName;
    navState.params = params;

    const url = buildUrl(screenName, params);
    window.history.replaceState({ screen: screenName, params }, '', url);

    showScreen(screenName, params, true);
    notifyNavListeners('replace', screenName, params);
  }

  /**
   * Build URL for a screen
   */
  function buildUrl(screenName, params) {
    let url = `#${screenName}`;
    const paramEntries = Object.entries(params);
    if (paramEntries.length > 0) {
      const searchParams = new URLSearchParams();
      paramEntries.forEach(([key, value]) => {
        searchParams.set(key, String(value));
      });
      url += '?' + searchParams.toString();
    }
    return url;
  }

  /**
   * Show a screen (hide others, show target)
   */
  function showScreen(screenName, params, animate) {
    // Find all screen containers
    const screenElements = document.querySelectorAll('[data-screen]');

    screenElements.forEach(element => {
      const name = element.getAttribute('data-screen');
      if (name === screenName) {
        element.style.display = '';
        element.setAttribute('data-active', 'true');

        // Trigger onMount if screen has lifecycle hooks
        const screen = screens.get(name);
        if (screen && screen.onMount) {
          screen.onMount(params);
        }
      } else {
        // Trigger onUnmount before hiding
        const screen = screens.get(name);
        if (screen && screen.onUnmount && element.getAttribute('data-active') === 'true') {
          screen.onUnmount();
        }

        element.style.display = 'none';
        element.removeAttribute('data-active');
      }
    });
  }

  /**
   * Add navigation change listener
   */
  function onNavChange(callback) {
    navState.listeners.push(callback);
    return () => {
      const index = navState.listeners.indexOf(callback);
      if (index > -1) navState.listeners.splice(index, 1);
    };
  }

  /**
   * Notify all navigation listeners
   */
  function notifyNavListeners(action, screen, params) {
    navState.listeners.forEach(cb => {
      try {
        cb({ action, screen, params });
      } catch (e) {
        console.error('[Clean Runtime] Navigation listener error:', e);
      }
    });
  }

  /**
   * Navigation API object
   */
  const nav = {
    go: navGo,
    back: navBack,
    replace: navReplace,
    get current() { return navState.current; },
    get params() { return { ...navState.params }; },
    get history() { return [...navState.history]; },
    onChange: onNavChange
  };

  // ============================================================================
  // LIFECYCLE HOOKS
  // ============================================================================

  /**
   * Register lifecycle hooks for a screen
   */
  function registerLifecycle(screenName, hooks) {
    const screen = screens.get(screenName);
    if (screen) {
      if (hooks.onMount) screen.onMount = hooks.onMount;
      if (hooks.onUnmount) screen.onUnmount = hooks.onUnmount;
      if (hooks.onUpdate) screen.onUpdate = hooks.onUpdate;
    }
  }

  // ============================================================================
  // APP STATE
  // ============================================================================

  /**
   * Global app state (shared across screens)
   */
  let appState = null;
  let appStateListeners = [];

  /**
   * Initialize app state
   */
  function initAppState(initialState) {
    appState = createReactiveState(initialState, () => {
      // Notify all listeners
      appStateListeners.forEach(cb => {
        try {
          cb({ ...appState });
        } catch (e) {
          console.error('[Clean Runtime] App state listener error:', e);
        }
      });

      // Re-render all active screens
      screens.forEach((screen, name) => {
        if (screen.element && screen.element.getAttribute('data-active') === 'true') {
          rerender(screen.element, name, screen.state);
        }
      });
    });
    return appState;
  }

  /**
   * Get app state
   */
  function getAppState() {
    return appState ? { ...appState } : null;
  }

  /**
   * Set app state
   */
  function setAppState(updates) {
    if (appState) {
      Object.assign(appState, updates);
    }
  }

  /**
   * Subscribe to app state changes
   */
  function onAppStateChange(callback) {
    appStateListeners.push(callback);
    return () => {
      const index = appStateListeners.indexOf(callback);
      if (index > -1) appStateListeners.splice(index, 1);
    };
  }

  // ============================================================================
  // THEMING SYSTEM (UI-40)
  // ============================================================================

  /**
   * Current theme name
   */
  let currentTheme = 'default';

  /**
   * Registered themes
   */
  const themes = new Map();

  /**
   * Register a theme
   */
  function registerTheme(name, cssVars) {
    themes.set(name, cssVars);
  }

  /**
   * Apply a theme
   */
  function applyTheme(themeName) {
    const root = document.documentElement;

    // Remove previous theme attribute
    root.removeAttribute('data-theme');

    // Apply new theme
    if (themeName !== 'default' && themes.has(themeName)) {
      root.setAttribute('data-theme', themeName);
    }

    currentTheme = themeName;

    // Dispatch theme change event
    document.dispatchEvent(new CustomEvent('clean:themeChange', {
      detail: { theme: themeName }
    }));

    console.log(`[Clean Runtime] Theme applied: ${themeName}`);
  }

  /**
   * Get current theme
   */
  function getTheme() {
    return currentTheme;
  }

  /**
   * Toggle between light and dark themes
   */
  function toggleDarkMode() {
    if (currentTheme === 'dark') {
      applyTheme('light');
    } else {
      applyTheme('dark');
    }
  }

  // ============================================================================
  // FORM VALIDATION (UI-41)
  // ============================================================================

  /**
   * Validation rules registry
   */
  const validators = {
    required: (value, _, message) => {
      if (!value || (typeof value === 'string' && value.trim() === '')) {
        return message || 'This field is required';
      }
      return null;
    },

    minLength: (value, min, message) => {
      if (value && value.length < parseInt(min)) {
        return message || `Must be at least ${min} characters`;
      }
      return null;
    },

    maxLength: (value, max, message) => {
      if (value && value.length > parseInt(max)) {
        return message || `Must be at most ${max} characters`;
      }
      return null;
    },

    email: (value, _, message) => {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (value && !emailRegex.test(value)) {
        return message || 'Invalid email address';
      }
      return null;
    },

    pattern: (value, pattern, message) => {
      if (value && !new RegExp(pattern).test(value)) {
        return message || 'Invalid format';
      }
      return null;
    },

    min: (value, min, message) => {
      if (value !== '' && parseFloat(value) < parseFloat(min)) {
        return message || `Must be at least ${min}`;
      }
      return null;
    },

    max: (value, max, message) => {
      if (value !== '' && parseFloat(value) > parseFloat(max)) {
        return message || `Must be at most ${max}`;
      }
      return null;
    },

    url: (value, _, message) => {
      try {
        if (value) new URL(value);
        return null;
      } catch {
        return message || 'Invalid URL';
      }
    },

    match: (value, fieldName, message) => {
      const otherField = document.querySelector(`[name="${fieldName}"]`);
      if (otherField && value !== otherField.value) {
        return message || `Must match ${fieldName}`;
      }
      return null;
    }
  };

  /**
   * Register a custom validator
   */
  function registerValidator(name, fn) {
    validators[name] = fn;
  }

  /**
   * Validate a single field
   */
  function validateField(element) {
    const rulesAttr = element.getAttribute('data-validate');
    if (!rulesAttr) return { valid: true, errors: [] };

    let rules;
    try {
      rules = JSON.parse(rulesAttr);
    } catch {
      return { valid: true, errors: [] };
    }

    const value = element.type === 'checkbox' ? element.checked : element.value;
    const errors = [];

    rules.forEach(rule => {
      const validator = validators[rule.type];
      if (validator) {
        const error = validator(value, rule.value, rule.message);
        if (error) errors.push(error);
      }
    });

    // Update UI
    const errorContainer = element.parentElement.querySelector('.validation-error');
    if (errors.length > 0) {
      element.classList.add('invalid');
      element.classList.remove('valid');
      element.setAttribute('aria-invalid', 'true');

      if (errorContainer) {
        errorContainer.textContent = errors[0];
        errorContainer.style.display = 'block';
      }
    } else {
      element.classList.remove('invalid');
      element.classList.add('valid');
      element.removeAttribute('aria-invalid');

      if (errorContainer) {
        errorContainer.textContent = '';
        errorContainer.style.display = 'none';
      }
    }

    return { valid: errors.length === 0, errors };
  }

  /**
   * Validate a form
   */
  function validateForm(formElement) {
    const fields = formElement.querySelectorAll('[data-validate]');
    let allValid = true;
    const allErrors = {};

    fields.forEach(field => {
      const result = validateField(field);
      if (!result.valid) {
        allValid = false;
        allErrors[field.name || field.id] = result.errors;
      }
    });

    return { valid: allValid, errors: allErrors };
  }

  /**
   * Setup validation listeners for a container
   */
  function setupValidation(container) {
    const fields = container.querySelectorAll('[data-validate]');

    fields.forEach(field => {
      // Validate on blur
      field.addEventListener('blur', () => validateField(field));

      // Validate on input (debounced)
      let timeout;
      field.addEventListener('input', () => {
        clearTimeout(timeout);
        timeout = setTimeout(() => validateField(field), 300);
      });
    });

    // Intercept form submission
    const forms = container.querySelectorAll('form');
    forms.forEach(form => {
      form.addEventListener('submit', (e) => {
        const result = validateForm(form);
        if (!result.valid) {
          e.preventDefault();
          // Focus first invalid field
          const firstInvalid = form.querySelector('.invalid');
          if (firstInvalid) firstInvalid.focus();
        }
      });
    });
  }

  // ============================================================================
  // ACCESSIBILITY (UI-42)
  // ============================================================================

  /**
   * Setup keyboard navigation for a container
   */
  function setupKeyboardNavigation(container) {
    // Arrow key navigation for lists/grids
    const navigables = container.querySelectorAll('[role="listbox"], [role="menu"], [role="tablist"]');

    navigables.forEach(nav => {
      const items = nav.querySelectorAll('[role="option"], [role="menuitem"], [role="tab"]');

      nav.addEventListener('keydown', (e) => {
        const currentIndex = Array.from(items).indexOf(document.activeElement);

        switch (e.key) {
          case 'ArrowDown':
          case 'ArrowRight':
            e.preventDefault();
            const nextIndex = (currentIndex + 1) % items.length;
            items[nextIndex]?.focus();
            break;
          case 'ArrowUp':
          case 'ArrowLeft':
            e.preventDefault();
            const prevIndex = (currentIndex - 1 + items.length) % items.length;
            items[prevIndex]?.focus();
            break;
          case 'Home':
            e.preventDefault();
            items[0]?.focus();
            break;
          case 'End':
            e.preventDefault();
            items[items.length - 1]?.focus();
            break;
        }
      });
    });
  }

  /**
   * Create focus trap for modals/dialogs
   */
  function createFocusTrap(container) {
    const focusables = container.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    if (focusables.length === 0) return null;

    const firstFocusable = focusables[0];
    const lastFocusable = focusables[focusables.length - 1];

    const trapHandler = (e) => {
      if (e.key === 'Tab') {
        if (e.shiftKey && document.activeElement === firstFocusable) {
          e.preventDefault();
          lastFocusable.focus();
        } else if (!e.shiftKey && document.activeElement === lastFocusable) {
          e.preventDefault();
          firstFocusable.focus();
        }
      }

      if (e.key === 'Escape') {
        container.dispatchEvent(new CustomEvent('clean:escape'));
      }
    };

    container.addEventListener('keydown', trapHandler);

    // Focus first element
    firstFocusable.focus();

    // Return cleanup function
    return () => container.removeEventListener('keydown', trapHandler);
  }

  /**
   * Announce message to screen readers
   */
  function announce(message, priority = 'polite') {
    let announcer = document.getElementById('clean-announcer');

    if (!announcer) {
      announcer = document.createElement('div');
      announcer.id = 'clean-announcer';
      announcer.setAttribute('aria-live', priority);
      announcer.setAttribute('aria-atomic', 'true');
      announcer.style.cssText = 'position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);border:0;';
      document.body.appendChild(announcer);
    }

    announcer.setAttribute('aria-live', priority);
    announcer.textContent = message;
  }

  /**
   * Setup skip links for keyboard users
   */
  function setupSkipLinks() {
    const mainContent = document.querySelector('main, [role="main"], #main-content');
    if (!mainContent) return;

    // Check if skip link already exists
    if (document.querySelector('.skip-link')) return;

    const skipLink = document.createElement('a');
    skipLink.href = '#main-content';
    skipLink.className = 'skip-link';
    skipLink.textContent = 'Skip to main content';
    skipLink.style.cssText = `
      position: absolute;
      top: -40px;
      left: 0;
      padding: 8px 16px;
      background: var(--color-primary, #4a90d9);
      color: white;
      z-index: 10000;
      transition: top 0.2s;
    `;

    skipLink.addEventListener('focus', () => {
      skipLink.style.top = '0';
    });

    skipLink.addEventListener('blur', () => {
      skipLink.style.top = '-40px';
    });

    mainContent.id = 'main-content';
    mainContent.setAttribute('tabindex', '-1');

    document.body.insertBefore(skipLink, document.body.firstChild);
  }

  /**
   * Check color contrast
   */
  function checkContrast(foreground, background) {
    const getLuminance = (rgb) => {
      const [r, g, b] = rgb.map(c => {
        c = c / 255;
        return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
      });
      return 0.2126 * r + 0.7152 * g + 0.0722 * b;
    };

    const parseColor = (color) => {
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      ctx.fillStyle = color;
      ctx.fillRect(0, 0, 1, 1);
      return ctx.getImageData(0, 0, 1, 1).data.slice(0, 3);
    };

    const l1 = getLuminance(parseColor(foreground));
    const l2 = getLuminance(parseColor(background));
    const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);

    return {
      ratio: ratio.toFixed(2),
      AA: ratio >= 4.5,
      AAA: ratio >= 7
    };
  }

  // ============================================================================
  // PUBLIC API
  // ============================================================================

  /**
   * Public API
   */
  window.CleanRuntime = {
    init,
    screens,
    hydrateScreen,
    getScreen: (name) => screens.get(name),
    getState: (name) => {
      const screen = screens.get(name);
      return screen ? { ...screen.state } : null;
    },
    setState: (name, updates) => {
      const screen = screens.get(name);
      if (screen) {
        Object.assign(screen.state, updates);
      }
    },
    // Navigation
    nav,
    // App state
    initAppState,
    getAppState,
    setAppState,
    onAppStateChange,
    // Lifecycle
    registerLifecycle,
    // Canvas
    canvases,
    initCanvases,
    getCanvas: (id) => canvases.get(id),
    pauseCanvas,
    resumeCanvas,
    // Theming
    themes,
    registerTheme,
    applyTheme,
    getTheme,
    toggleDarkMode,
    // Validation
    validators,
    registerValidator,
    validateField,
    validateForm,
    setupValidation,
    // Accessibility
    setupKeyboardNavigation,
    createFocusTrap,
    announce,
    setupSkipLinks,
    checkContrast
  };

  // Make nav globally accessible for convenience
  window.nav = nav;

  // Make canvas API globally accessible
  window.canvas = {
    get: (id) => canvases.get(id)?.api,
    pause: pauseCanvas,
    resume: resumeCanvas
  };

  // Auto-initialize when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

})();
