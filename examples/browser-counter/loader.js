// Clean Language Template-Based Browser Runtime
// Loads HTML template and binds WASM state

(async function() {
	const script = document.currentScript;
	const wasmPath = script.dataset.wasm || 'app.wasm';
	const templatePath = script.dataset.template || 'template.html';
	const appEl = document.getElementById('app');

	let memory, wasm, template;
	let heapPtr = 65536;
	let initialHeapPtr = 65536;

	// String utilities
	function readString(ptr) {
		if (!ptr || ptr < 0 || ptr >= memory.buffer.byteLength) return '';
		try {
			const view = new DataView(memory.buffer);
			const len = view.getUint32(ptr, true);
			if (len < 0 || len > 1000000 || ptr + 4 + len > memory.buffer.byteLength) return '';
			return new TextDecoder().decode(new Uint8Array(memory.buffer, ptr + 4, len));
		} catch (e) { return ''; }
	}

	function writeString(str) {
		const bytes = new TextEncoder().encode(str);
		const ptr = heapPtr;
		const view = new DataView(memory.buffer);
		view.setUint32(ptr, bytes.length, true);
		new Uint8Array(memory.buffer, ptr + 4, bytes.length).set(bytes);
		heapPtr += 4 + bytes.length + (4 - (bytes.length % 4)) % 4;
		return ptr;
	}

	// WASM imports
	const imports = {
		env: {
			// I/O
			print: (ptr) => console.log(readString(ptr)),
			printl: (ptr) => console.log(readString(ptr)),
			input: (p) => writeString(prompt(readString(p)) || ''),
			input_integer: (p) => writeString(prompt(readString(p)) || '0'),
			input_float: (p) => parseFloat(prompt(readString(p)) || '0'),
			input_yesno: (p) => confirm(readString(p)) ? 1 : 0,
			input_range: (p, min, max) => {
				const val = parseInt(prompt(readString(p)) || '0');
				return Math.max(min, Math.min(max, val));
			},
			// String operations
			string_to_float: (p) => parseFloat(readString(p)) || 0,
			float_to_string: (f) => writeString(f.toString()),
			'string.concat': (p1, p2) => writeString(readString(p1) + readString(p2)),
			'string.split': () => 0,
			string_trim: (p) => writeString(readString(p).trim()),
			string_trim_start: (p) => writeString(readString(p).trimStart()),
			string_trim_end: (p) => writeString(readString(p).trimEnd()),
			string_compare: (p1, p2) => readString(p1).localeCompare(readString(p2)),
			string_replace: (p, old, newStr) => writeString(readString(p).replace(readString(old), readString(newStr))),
			// Math
			math_pow: Math.pow, math_sin: Math.sin, math_cos: Math.cos, math_tan: Math.tan,
			math_sqrt: Math.sqrt, math_abs: Math.abs, math_floor: Math.floor, math_ceil: Math.ceil,
			math_round: Math.round, math_min: Math.min, math_max: Math.max,
			math_log: Math.log, math_ln: Math.log, math_log10: Math.log10, math_log2: Math.log2,
			math_exp: Math.exp, math_exp2: (x) => Math.pow(2, x), math_asin: Math.asin, math_acos: Math.acos,
			math_atan: Math.atan, math_atan2: Math.atan2,
			math_sinh: Math.sinh, math_cosh: Math.cosh, math_tanh: Math.tanh,
			math_asinh: Math.asinh, math_acosh: Math.acosh, math_atanh: Math.atanh,
			math_cbrt: Math.cbrt, math_hypot: Math.hypot, math_sign: Math.sign, math_trunc: Math.trunc,
			math_random: Math.random,
			math_pi: () => Math.PI, math_e: () => Math.E,
			// HTTP (stubs for browser)
			http_get: () => 0, http_post: () => 0, http_put: () => 0, http_patch: () => 0,
			http_delete: () => 0, http_head: () => 0, http_options: () => 0,
			http_get_with_headers: () => 0, http_post_with_headers: () => 0,
			http_post_json: () => 0, http_put_json: () => 0, http_patch_json: () => 0,
			http_post_form: () => 0,
			http_set_user_agent: () => {}, http_set_timeout: () => {}, http_set_max_redirects: () => {},
			http_enable_cookies: () => {},
			http_get_response_code: () => 0, http_get_response_headers: () => 0,
			http_encode_url: (p) => writeString(encodeURIComponent(readString(p))),
			http_decode_url: (p) => writeString(decodeURIComponent(readString(p))),
			http_build_query: () => 0,
			// File (stubs for browser)
			file_read: () => 0, file_write: () => 0, file_exists: () => 0, file_delete: () => 0, file_append: () => 0
		},
		memory_runtime: {
			mem_alloc: (t, s) => { const p = heapPtr; heapPtr += s + ((4-(s%4))%4); return p; },
			mem_retain: () => {}, mem_release: () => {},
			mem_scope_push: () => {}, mem_scope_pop: () => {}
		}
	};

	// Get state value from WASM
	function getState(name) {
		heapPtr = initialHeapPtr; // Reset heap for each getter call
		const getter = wasm.exports['get_' + name];
		if (!getter) return undefined;
		const result = getter();
		// Check if it's a string pointer
		if (typeof result === 'number' && result > 1000) {
			return readString(result);
		}
		return result;
	}

	// Render template with current state
	function render() {
		if (!template) return;

		const state = {
			count: getState('count'),
			clicks: getState('clicks'),
			progress: getState('progress'),
			darkMode: getState('darkMode'),
			statusMsg: getState('statusMsg'),
			activeTab: getState('activeTab'),
			userName: getState('userName'),
			showGreeting: getState('showGreeting'),
			theme: getState('theme'),
			countColor: getState('countColor')
		};

		// Clone template
		const html = template.cloneNode(true);

		// Apply data-class (theme class on root)
		html.querySelectorAll('[data-class]').forEach(el => {
			const attr = el.dataset.class;
			const value = state[attr];
			if (value) {
				el.className = el.className.replace(/\b(dark|light)\b/g, '').trim();
				el.classList.add(value);
			}
		});

		// Apply data-value (text content)
		html.querySelectorAll('[data-value]').forEach(el => {
			const key = el.dataset.value;
			const value = state[key];
			if (value !== undefined) {
				el.textContent = value;
			}
		});

		// Apply data-style (dynamic styles)
		html.querySelectorAll('[data-style]').forEach(el => {
			const styleAttr = el.dataset.style;
			// Parse "property:stateKey" or "property:stateKey%"
			const match = styleAttr.match(/^(\w+):(\w+)(%?)$/);
			if (match) {
				const [, prop, key, suffix] = match;
				const value = state[key];
				if (value !== undefined) {
					el.style[prop] = value + suffix;
				}
			}
		});

		// Apply data-show (conditional visibility)
		html.querySelectorAll('[data-show]').forEach(el => {
			const condition = el.dataset.show;
			let visible = false;
			if (condition.startsWith('!')) {
				visible = !state[condition.slice(1)];
			} else {
				visible = !!state[condition];
			}
			el.style.display = visible ? '' : 'none';
		});

		// Apply data-active (active state for tabs)
		html.querySelectorAll('[data-active]').forEach(el => {
			const [key, value] = el.dataset.active.split(':');
			const isActive = state[key] === parseInt(value);
			el.classList.toggle('active', isActive);
		});

		// Apply data-tab (show/hide tab panels)
		html.querySelectorAll('[data-tab]').forEach(el => {
			const tabIndex = parseInt(el.dataset.tab);
			el.style.display = state.activeTab === tabIndex ? 'block' : 'none';
		});

		// Restore input values
		html.querySelectorAll('[data-bind]').forEach(el => {
			const key = el.dataset.bind;
			const value = state[key];
			if (value !== undefined && el.tagName === 'INPUT') {
				el.value = value;
			}
		});

		// Update DOM
		appEl.innerHTML = '';
		appEl.appendChild(html);

		// Re-init canvas if visible
		if (state.activeTab === 2) {
			setTimeout(initCanvas, 50);
		}
	}

	// Event handling
	function setupEvents() {
		appEl.addEventListener('click', (e) => {
			const btn = e.target.closest('[data-event]');
			if (btn) {
				const evtName = btn.dataset.event;
				const handler = wasm.exports['handle_event_' + evtName];
				if (handler) {
					handler();
					render();
					// Canvas-specific
					if (evtName === 'moreParticles') addParticles(20);
					if (evtName === 'fewerParticles') removeParticles(20);
					if (evtName === 'resetCanvas') resetParticles();
				}
			}
		});

		appEl.addEventListener('input', (e) => {
			const bind = e.target.dataset.bind;
			if (bind) {
				heapPtr = initialHeapPtr;
				const setter = wasm.exports['set_' + bind];
				if (setter) setter(writeString(e.target.value));
			}
		});

		appEl.addEventListener('blur', (e) => {
			if (e.target.dataset.bind) render();
		}, true);
	}

	// Particle Animation System
	let particles = [];
	let canvas = null;
	let ctx = null;
	let animationId = null;
	let particleCount = 50;
	const colors = ['#6366f1', '#8b5cf6', '#06b6d4', '#10b981', '#f59e0b', '#ef4444'];

	class Particle {
		constructor() {
			this.x = Math.random() * (canvas?.width || 600);
			this.y = Math.random() * (canvas?.height || 300);
			this.vx = (Math.random() - 0.5) * 2;
			this.vy = (Math.random() - 0.5) * 2;
			this.radius = Math.random() * 3 + 1;
			this.color = colors[Math.floor(Math.random() * colors.length)];
			this.alpha = Math.random() * 0.5 + 0.5;
			this.pulse = Math.random() * Math.PI * 2;
		}

		update() {
			this.x += this.vx;
			this.y += this.vy;
			this.pulse += 0.05;
			if (this.x < 0 || this.x > canvas.width) this.vx *= -1;
			if (this.y < 0 || this.y > canvas.height) this.vy *= -1;
			this.x = Math.max(0, Math.min(canvas.width, this.x));
			this.y = Math.max(0, Math.min(canvas.height, this.y));
		}

		draw() {
			const r = this.radius + Math.sin(this.pulse) * 0.5;
			ctx.beginPath();
			ctx.arc(this.x, this.y, r, 0, Math.PI * 2);
			ctx.fillStyle = this.color;
			ctx.globalAlpha = this.alpha;
			ctx.fill();
			ctx.globalAlpha = 1;
		}
	}

	function initCanvas() {
		canvas = document.getElementById('particleCanvas');
		if (!canvas) return;
		ctx = canvas.getContext('2d');
		canvas.width = canvas.offsetWidth || 600;
		canvas.height = 300;
		if (particles.length === 0) {
			for (let i = 0; i < particleCount; i++) particles.push(new Particle());
		}
		if (!animationId) animate();
	}

	function animate() {
		if (!canvas || !ctx) { animationId = null; return; }
		ctx.clearRect(0, 0, canvas.width, canvas.height);

		// Draw connections
		for (let i = 0; i < particles.length; i++) {
			for (let j = i + 1; j < particles.length; j++) {
				const dx = particles[i].x - particles[j].x;
				const dy = particles[i].y - particles[j].y;
				const dist = Math.sqrt(dx * dx + dy * dy);
				if (dist < 100) {
					ctx.beginPath();
					ctx.moveTo(particles[i].x, particles[i].y);
					ctx.lineTo(particles[j].x, particles[j].y);
					ctx.strokeStyle = particles[i].color;
					ctx.globalAlpha = (1 - dist / 100) * 0.3;
					ctx.stroke();
					ctx.globalAlpha = 1;
				}
			}
		}

		particles.forEach(p => { p.update(); p.draw(); });
		animationId = requestAnimationFrame(animate);
	}

	function addParticles(n) {
		for (let i = 0; i < n; i++) particles.push(new Particle());
	}

	function removeParticles(n) {
		particles.splice(0, Math.min(n, particles.length - 5));
	}

	function resetParticles() {
		particles = [];
		for (let i = 0; i < 50; i++) particles.push(new Particle());
	}

	// Initialize
	try {
		// Load template
		const templateResp = await fetch(templatePath);
		const templateHtml = await templateResp.text();
		const templateDoc = new DOMParser().parseFromString(templateHtml, 'text/html');
		template = templateDoc.body.firstElementChild;

		// Load WASM
		const wasmResp = await fetch(wasmPath);
		const module = await WebAssembly.instantiate(await wasmResp.arrayBuffer(), imports);
		wasm = module.instance;
		memory = wasm.exports.memory;

		if (wasm.exports.__heap_ptr) {
			initialHeapPtr = wasm.exports.__heap_ptr.value || 65536;
			heapPtr = initialHeapPtr;
		}

		if (wasm.exports._start) wasm.exports._start();

		render();
		setupEvents();

		// Periodic canvas check
		setInterval(() => {
			if (document.getElementById('particleCanvas') && !animationId) initCanvas();
		}, 500);

	} catch (err) {
		console.error('Error:', err);
		appEl.innerHTML = `<div style="color:#ef4444;padding:2rem;font-family:monospace;background:#1a1a2e;border-radius:16px;margin:2rem;">
			<h2>Failed to load application</h2>
			<pre>${err.message}\n${err.stack || ''}</pre>
		</div>`;
	}
})();
