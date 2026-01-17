// Frame UI Runtime Loader
// This is framework code - developers write only Clean Language

(async function() {
	const script = document.currentScript;
	const wasmPath = script.dataset.wasm || 'frontend.wasm';
	const appEl = document.getElementById('app');
	const screenName = appEl?.dataset.screen || 'Main';

	// Host bridge functions for WASM
	const bridge = {
		env: {
			print: (ptr, len) => console.log(readString(ptr, len)),
			printl: (ptr, len) => console.log(readString(ptr, len)),
			'string.concat': (ptr1, len1, ptr2, len2) => {
				const s1 = readString(ptr1, len1);
				const s2 = readString(ptr2, len2);
				return writeString(s1 + s2);
			},
			// Memory management
			mem_alloc: (size) => memory.alloc(size),
			mem_retain: (ptr) => {},
			mem_release: (ptr) => {},
			mem_scope_push: () => {},
			mem_scope_pop: () => {},
		}
	};

	let memory, instance;
	let heapPtr = 4096;

	function readString(ptr, len) {
		const bytes = new Uint8Array(memory.buffer, ptr, len);
		return new TextDecoder().decode(bytes);
	}

	function writeString(str) {
		const bytes = new TextEncoder().encode(str);
		const ptr = heapPtr;
		const view = new Uint8Array(memory.buffer, ptr, bytes.length + 8);
		// Write length prefix (4 bytes) + capacity (4 bytes) + data
		new DataView(memory.buffer).setUint32(ptr, bytes.length, true);
		new DataView(memory.buffer).setUint32(ptr + 4, bytes.length, true);
		view.set(bytes, 8);
		heapPtr += bytes.length + 8 + (8 - (bytes.length % 8)); // Align
		return ptr;
	}

	try {
		const response = await fetch(wasmPath);
		const bytes = await response.arrayBuffer();
		const module = await WebAssembly.instantiate(bytes, bridge);
		instance = module.instance;
		memory = instance.exports.memory;

		// Get heap pointer from WASM if available
		if (instance.exports.__heap_ptr) {
			heapPtr = instance.exports.__heap_ptr.value;
		}

		// Call render function if it exists
		if (instance.exports.render) {
			const htmlPtr = instance.exports.render();
			if (htmlPtr && appEl) {
				// Read HTML string from WASM memory
				const len = new DataView(memory.buffer).getUint32(htmlPtr, true);
				const html = readString(htmlPtr + 8, len);
				appEl.innerHTML = html;
			}
		}

		// Setup event handlers
		setupEventHandlers(appEl, instance);

	} catch (err) {
		console.error('Frame UI Loader Error:', err);
		if (appEl) {
			appEl.innerHTML = '<div style="color:red;padding:20px;">Failed to load application</div>';
		}
	}

	function setupEventHandlers(root, wasm) {
		// Handle click events
		root.addEventListener('click', (e) => {
			const btn = e.target.closest('[data-event]');
			if (btn) {
				const eventId = btn.dataset.event;
				if (wasm.exports['handle_event_' + eventId]) {
					wasm.exports['handle_event_' + eventId]();
					rerender();
				}
			}
		});

		// Handle input events
		root.addEventListener('input', (e) => {
			const input = e.target;
			const bind = input.dataset.bind;
			if (bind && wasm.exports['set_' + bind]) {
				const ptr = writeString(input.value);
				wasm.exports['set_' + bind](ptr);
				rerender();
			}
		});

		function rerender() {
			if (wasm.exports.render) {
				const htmlPtr = wasm.exports.render();
				if (htmlPtr) {
					const len = new DataView(memory.buffer).getUint32(htmlPtr, true);
					const html = readString(htmlPtr + 8, len);
					// Preserve focus during rerender
					const focused = document.activeElement;
					const focusId = focused?.id;
					const focusPos = focused?.selectionStart;
					root.innerHTML = html;
					if (focusId) {
						const el = document.getElementById(focusId);
						if (el) {
							el.focus();
							if (focusPos !== undefined && el.setSelectionRange) {
								el.setSelectionRange(focusPos, focusPos);
							}
						}
					}
				}
			}
		}
	}
})();
