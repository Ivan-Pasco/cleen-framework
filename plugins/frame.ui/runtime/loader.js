// Frame UI Runtime Loader
// WASM loader with full browser API bridge
// Version: 2.2.0
//
// All interactivity uses _ui_on_event delegation + targeted DOM updates.
// WASM _start() registers handlers, handlers update DOM via bridge functions.

(async function() {
	const script = document.currentScript;
	const wasmPath = script.dataset.wasm || 'frontend.wasm';

	const WASM_PAGE_SIZE = 65536;

	// --- State ---
	let memory, instance;
	// heapPtr MUST be initialized from the WASM module's __heap_ptr export.
	// See platform-architecture/MEMORY_POLICY.md §7.2.
	let heapPtr = -1;
	let currentEvent = null;
	let currentEventTarget = null;
	const componentRegistry = new Map();
	const slotStore = new Map();
	const stateStore = new Map();
	const registeredEventTypes = new Set();
	const eventHandlers = new Map(); // "selector\0eventType" -> [handlerIdx, ...]

	// --- Memory observability (MEMORY_POLICY.md §9.3) ---

	const memStats = {
		maxPages: null,
		growCount: 0,
		lastBufferSize: 0,
	};

	function checkMemoryGrowth() {
		if (!memory) return;
		const size = memory.buffer.byteLength;
		if (size !== memStats.lastBufferSize) {
			const fromPages = memStats.lastBufferSize / WASM_PAGE_SIZE;
			const toPages = size / WASM_PAGE_SIZE;
			const fromMB = (memStats.lastBufferSize / (1024 * 1024)).toFixed(1);
			const toMB = (size / (1024 * 1024)).toFixed(1);
			console.warn(`[clean] Memory grew: ${fromPages} → ${toPages} pages (${fromMB} MB → ${toMB} MB)`);
			memStats.growCount++;
			memStats.lastBufferSize = size;
		}
	}

	function memoryStats() {
		const size = memory ? memory.buffer.byteLength : 0;
		return {
			currentPages: size / WASM_PAGE_SIZE,
			currentBytes: size,
			maxPages: memStats.maxPages,
			growCount: memStats.growCount,
			heapPtr: heapPtr,
		};
	}

	// --- WASM memory-section parsing (for maxPages) ---

	function readULEB128(bytes, ref) {
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

	function readWasmString(bytes, ref) {
		const len = readULEB128(bytes, ref);
		ref.offset += len;
	}

	function parseMaxPages(wasmBuffer) {
		const bytes = new Uint8Array(wasmBuffer);
		if (bytes.length < 8) return null;
		if (bytes[0] !== 0x00 || bytes[1] !== 0x61 || bytes[2] !== 0x73 || bytes[3] !== 0x6d) return null;
		const ref = { offset: 8 };
		while (ref.offset < bytes.length) {
			const sectionId = bytes[ref.offset++];
			const sectionSize = readULEB128(bytes, ref);
			const sectionEnd = ref.offset + sectionSize;
			try {
				if (sectionId === 5) {
					const count = readULEB128(bytes, ref);
					if (count > 0) {
						const flags = readULEB128(bytes, ref);
						readULEB128(bytes, ref); // initial
						if (flags & 0x01) return readULEB128(bytes, ref);
						return null;
					}
				}
			} catch (_) { /* fall through */ }
			ref.offset = sectionEnd;
		}
		return null;
	}

	// --- Memory Helpers ---

	function readString(ptr, len) {
		checkMemoryGrowth();
		const bytes = new Uint8Array(memory.buffer, ptr, len);
		return new TextDecoder().decode(bytes);
	}

	function writeString(str) {
		checkMemoryGrowth();
		const bytes = new TextEncoder().encode(str);
		const ptr = heapPtr;
		const view = new Uint8Array(memory.buffer, ptr, bytes.length + 8);
		new DataView(memory.buffer).setUint32(ptr, bytes.length, true);
		new DataView(memory.buffer).setUint32(ptr + 4, bytes.length, true);
		view.set(bytes, 8);
		heapPtr += bytes.length + 8 + (8 - (bytes.length % 8));
		return ptr;
	}

	// --- Event Delegation ---

	function ensureDocumentListener(eventType) {
		if (registeredEventTypes.has(eventType)) return;
		registeredEventTypes.add(eventType);

		document.addEventListener(eventType, (e) => {
			for (const [key, handlers] of eventHandlers) {
				const sep = key.indexOf('\0');
				const selector = key.substring(0, sep);
				const type = key.substring(sep + 1);
				if (type !== eventType) continue;

				const target = e.target.closest(selector);
				if (target) {
					handlers.forEach(handlerIdx => {
						currentEvent = e;
						currentEventTarget = target;
						const fn = instance.exports['handle_event_' + handlerIdx];
						if (fn) fn();
						currentEvent = null;
						currentEventTarget = null;
					});
				}
			}
		});
	}

	// --- Bridge Object ---

	const bridge = {
		memory_runtime: {
			// Memory management — spec: platform-architecture/HOST_BRIDGE.md
			mem_alloc: (size) => {
				const ptr = heapPtr;
				heapPtr += size + (8 - (size % 8));
				return ptr;
			},
			mem_retain: (ptr) => {},
			mem_release: (ptr) => {},
			mem_scope_push: () => {},
			mem_scope_pop: () => {},
		},
		env: {
			// Console
			print: (ptr, len) => console.log(readString(ptr, len)),
			printl: (ptr, len) => console.log(readString(ptr, len)),
			'string.concat': (ptr1, len1, ptr2, len2) => {
				return writeString(readString(ptr1, len1) + readString(ptr2, len2));
			},

			// ========== HTML Rendering ==========

			_html_escape: (strPtr, strLen) => {
				const str = readString(strPtr, strLen);
				const escaped = str
					.replace(/&/g, '&amp;')
					.replace(/</g, '&lt;')
					.replace(/>/g, '&gt;')
					.replace(/"/g, '&quot;')
					.replace(/'/g, '&#039;');
				return writeString(escaped);
			},

			_html_raw: (strPtr, strLen) => {
				return writeString(readString(strPtr, strLen));
			},

			// ========== Component Registry ==========

			_ui_register_component: (tagPtr, tagLen, classPtr, classLen) => {
				componentRegistry.set(readString(tagPtr, tagLen), readString(classPtr, classLen));
				return 0;
			},

			_ui_get_component: (tagPtr, tagLen) => {
				return writeString(componentRegistry.get(readString(tagPtr, tagLen)) || '');
			},

			// ========== Slot Management ==========

			_ui_set_slot: (namePtr, nameLen, contentPtr, contentLen) => {
				slotStore.set(readString(namePtr, nameLen), readString(contentPtr, contentLen));
				return 0;
			},

			_ui_get_slot: (namePtr, nameLen) => {
				return writeString(slotStore.get(readString(namePtr, nameLen)) || '');
			},

			// ========== Event Handler Registration ==========

			_ui_on_event: (selectorPtr, selectorLen, eventTypePtr, eventTypeLen, handlerIdx) => {
				const selector = readString(selectorPtr, selectorLen);
				const eventType = readString(eventTypePtr, eventTypeLen);
				const key = selector + '\0' + eventType;

				if (!eventHandlers.has(key)) {
					eventHandlers.set(key, []);
				}
				eventHandlers.get(key).push(handlerIdx);
				ensureDocumentListener(eventType);
				return 0;
			},

			// ========== State Management ==========

			_ui_set_state: (idPtr, idLen, jsonPtr, jsonLen) => {
				stateStore.set(readString(idPtr, idLen), readString(jsonPtr, jsonLen));
				return 0;
			},

			_ui_get_state: (idPtr, idLen) => {
				return writeString(stateStore.get(readString(idPtr, idLen)) || '{}');
			},

			// ========== DOM Manipulation ==========

			_ui_update_element: (selectorPtr, selectorLen, contentPtr, contentLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) { el.innerHTML = readString(contentPtr, contentLen); return 0; }
				return -1;
			},

			_ui_update_attr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) { el.setAttribute(readString(attrPtr, attrLen), readString(valPtr, valLen)); return 0; }
				return -1;
			},

			// ========== Form Binding ==========

			_ui_bind_input: (selectorPtr, selectorLen, pathPtr, pathLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const path = readString(pathPtr, pathLen);
				const el = document.querySelector(selector);
				if (!el) return -1;

				el.addEventListener('input', () => {
					stateStore.set(path, JSON.stringify(el.value));
					const setterName = 'set_' + path.replace(/\./g, '_');
					if (instance.exports[setterName]) {
						instance.exports[setterName](writeString(el.value));
					}
				});
				return 0;
			},

			// ========== Validation ==========

			_ui_validate: (valuePtr, valueLen, rulePtr, ruleLen) => {
				const value = readString(valuePtr, valueLen);
				const rule = readString(rulePtr, ruleLen);
				let error = '';
				switch (rule) {
					case 'required': if (!value.trim()) error = 'This field is required'; break;
					case 'email': if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) error = 'Invalid email'; break;
					case 'url': try { new URL(value); } catch { error = 'Invalid URL'; } break;
				}
				return writeString(error);
			},

			// ========== Event Handler Context ==========

			_ui_event_attr: (attrPtr, attrLen) => {
				if (!currentEventTarget) return writeString('');
				return writeString(currentEventTarget.getAttribute(readString(attrPtr, attrLen)) || '');
			},

			_ui_event_value: () => {
				if (!currentEventTarget) return writeString('');
				if (currentEventTarget.tagName === 'INPUT' || currentEventTarget.tagName === 'TEXTAREA' || currentEventTarget.tagName === 'SELECT') {
					return writeString(currentEventTarget.value || '');
				}
				return writeString(currentEventTarget.textContent || '');
			},

			_ui_event_closest_attr: (selectorPtr, selectorLen, attrPtr, attrLen) => {
				if (!currentEventTarget) return writeString('');
				const el = currentEventTarget.closest(readString(selectorPtr, selectorLen));
				if (!el) return writeString('');
				return writeString(el.getAttribute(readString(attrPtr, attrLen)) || '');
			},

			_ui_event_type: () => {
				if (!currentEvent) return writeString('');
				return writeString(currentEvent.type);
			},

			// ========== Clipboard ==========

			_ui_clipboard_write: (textPtr, textLen) => {
				navigator.clipboard.writeText(readString(textPtr, textLen)).catch(() => {});
				return 0;
			},

			// ========== URL / Location ==========

			_ui_location_href: (urlPtr, urlLen) => {
				window.location.href = readString(urlPtr, urlLen);
				return 0;
			},

			_ui_location_query: (paramPtr, paramLen) => {
				const value = new URLSearchParams(window.location.search).get(readString(paramPtr, paramLen)) || '';
				return writeString(value);
			},

			_ui_location_path: () => {
				return writeString(window.location.pathname);
			},

			// ========== DOM Query (single element) ==========

			_ui_get_text: (selectorPtr, selectorLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				return writeString(el ? (el.textContent || '') : '');
			},

			_ui_get_attr: (selectorPtr, selectorLen, attrPtr, attrLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				return writeString(el ? (el.getAttribute(readString(attrPtr, attrLen)) || '') : '');
			},

			_ui_toggle_class: (selectorPtr, selectorLen, classPtr, classLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.classList.toggle(readString(classPtr, classLen));
				return 0;
			},

			_ui_add_class: (selectorPtr, selectorLen, classPtr, classLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.classList.add(readString(classPtr, classLen));
				return 0;
			},

			_ui_remove_class: (selectorPtr, selectorLen, classPtr, classLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.classList.remove(readString(classPtr, classLen));
				return 0;
			},

			_ui_set_style: (selectorPtr, selectorLen, propPtr, propLen, valPtr, valLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.style[readString(propPtr, propLen)] = readString(valPtr, valLen);
				return 0;
			},

			_ui_update_element_self: (contentPtr, contentLen) => {
				if (!currentEventTarget) return -1;
				currentEventTarget.textContent = readString(contentPtr, contentLen);
				return 0;
			},

			// ========== DOM Batch (querySelectorAll) ==========

			_ui_query_set_style: (selectorPtr, selectorLen, propPtr, propLen, valPtr, valLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const prop = readString(propPtr, propLen);
				const val = readString(valPtr, valLen);
				document.querySelectorAll(selector).forEach(el => { el.style[prop] = val; });
				return 0;
			},

			_ui_query_set_attr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const attr = readString(attrPtr, attrLen);
				const val = readString(valPtr, valLen);
				document.querySelectorAll(selector).forEach(el => el.setAttribute(attr, val));
				return 0;
			},

			_ui_query_add_class: (selectorPtr, selectorLen, classPtr, classLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const cls = readString(classPtr, classLen);
				document.querySelectorAll(selector).forEach(el => el.classList.add(cls));
				return 0;
			},

			_ui_query_remove_class: (selectorPtr, selectorLen, classPtr, classLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const cls = readString(classPtr, classLen);
				document.querySelectorAll(selector).forEach(el => el.classList.remove(cls));
				return 0;
			},

			_ui_filter_by_attr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const attr = readString(attrPtr, attrLen);
				const val = readString(valPtr, valLen);
				document.querySelectorAll(selector).forEach(el => {
					if (val === '*' || el.getAttribute(attr) === val) {
						el.style.display = '';
					} else {
						el.style.display = 'none';
					}
				});
				return 0;
			},

			_ui_filter_by_text: (selectorPtr, selectorLen, nameAttrPtr, nameAttrLen, descAttrPtr, descAttrLen, queryPtr, queryLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const nameAttr = readString(nameAttrPtr, nameAttrLen);
				const descAttr = readString(descAttrPtr, descAttrLen);
				const query = readString(queryPtr, queryLen).toLowerCase();
				document.querySelectorAll(selector).forEach(el => {
					if (query.length === 0) {
						el.style.display = '';
					} else {
						const name = (el.getAttribute(nameAttr) || '').toLowerCase();
						const desc = (el.getAttribute(descAttr) || '').toLowerCase();
						el.style.display = (name.indexOf(query) !== -1 || desc.indexOf(query) !== -1) ? '' : 'none';
					}
				});
				return 0;
			},

			// ========== IntersectionObserver ==========

			_ui_observe_visible: (selectorPtr, selectorLen, classPtr, classLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const cls = readString(classPtr, classLen);
				if (!('IntersectionObserver' in window)) {
					document.querySelectorAll(selector).forEach(el => el.classList.add(cls));
					return 0;
				}
				const observer = new IntersectionObserver((entries) => {
					entries.forEach(entry => {
						if (entry.isIntersecting) {
							entry.target.classList.add(cls);
							observer.unobserve(entry.target);
						}
					});
				}, { threshold: 0.1 });
				document.querySelectorAll(selector).forEach(el => observer.observe(el));
				return 0;
			},

			// ========== Timers ==========

			_ui_set_timeout: (handlerIdx, delayMs) => {
				setTimeout(() => {
					const fn = instance.exports['handle_event_' + handlerIdx];
					if (fn) {
						currentEvent = null;
						currentEventTarget = null;
						fn();
					}
				}, delayMs);
				return 0;
			},

			// ========== Theming ==========

			_ui_inject_head_css: (cssPtr, cssLen) => {
				const css = readString(cssPtr, cssLen);
				const style = document.createElement('style');
				style.textContent = css;
				document.head.appendChild(style);
				return 0;
			},

			// ========== Layout Loading ==========

			_ui_load_layout: (namePtr, nameLen) => {
				const name = readString(namePtr, nameLen);
				// Layout loading requires server-side fetch; return empty on client
				console.warn('_ui_load_layout is server-side only, called with:', name);
				return writeString('');
			},

			// ========== Form Helpers ==========

			_ui_input_value: (selectorPtr, selectorLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (!el) return writeString('');
				return writeString(el.value || el.textContent || '');
			},

			_ui_form_json: (selectorPtr, selectorLen) => {
				const form = document.querySelector(readString(selectorPtr, selectorLen));
				if (!form) return writeString('{}');
				const data = {};
				form.querySelectorAll('[name]').forEach(el => {
					if (el.type === 'checkbox') {
						data[el.name] = el.checked;
					} else if (el.type === 'radio') {
						if (el.checked) data[el.name] = el.value;
					} else {
						data[el.name] = el.value;
					}
				});
				return writeString(JSON.stringify(data));
			},

			_ui_form_data: (selectorPtr, selectorLen) => {
				const form = document.querySelector(readString(selectorPtr, selectorLen));
				if (!form) return writeString('');
				const parts = [];
				form.querySelectorAll('[name]').forEach(el => {
					if (el.type === 'checkbox') {
						if (el.checked) parts.push(encodeURIComponent(el.name) + '=' + encodeURIComponent(el.value || 'on'));
					} else if (el.type === 'radio') {
						if (el.checked) parts.push(encodeURIComponent(el.name) + '=' + encodeURIComponent(el.value));
					} else {
						parts.push(encodeURIComponent(el.name) + '=' + encodeURIComponent(el.value));
					}
				});
				return writeString(parts.join('&'));
			},

			_ui_checked: (selectorPtr, selectorLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (!el) return 0;
				return el.checked ? 1 : 0;
			},

			_ui_set_input: (selectorPtr, selectorLen, valPtr, valLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (!el) return -1;
				el.value = readString(valPtr, valLen);
				return 0;
			},

		}
	};

	// --- WASM Loading ---

	try {
		const response = await fetch(wasmPath);
		const bytes = await response.arrayBuffer();

		memStats.maxPages = parseMaxPages(bytes);

		const module = await WebAssembly.instantiate(bytes, bridge);
		instance = module.instance;
		memory = instance.exports.memory;
		memStats.lastBufferSize = memory.buffer.byteLength;

		// __heap_ptr is authoritative per MEMORY_POLICY.md §7.2.
		// A missing export means the module was compiled without the runtime heap setup —
		// falling back to a hardcoded offset would corrupt the data section.
		const heapPtrExport = instance.exports.__heap_ptr;
		if (!heapPtrExport || typeof heapPtrExport.value !== 'number') {
			throw new Error(
				'Frame UI: WASM module missing __heap_ptr export. ' +
				'Recompile with a compiler that emits the runtime heap pointer.'
			);
		}
		heapPtr = heapPtrExport.value;

		if (typeof window !== 'undefined') {
			if (!window.__cleanRuntime) {
				window.__cleanRuntime = {
					_providers: [],
					memoryStats() {
						const p = this._providers[this._providers.length - 1];
						return p ? p() : null;
					}
				};
			}
			window.__cleanRuntime._providers.push(memoryStats);
		}

		checkMemoryGrowth();

		// Call _start — registers event handlers via _ui_on_event
		if (instance.exports._start) {
			instance.exports._start();
		} else if (instance.exports.start) {
			instance.exports.start();
		}

		checkMemoryGrowth();

	} catch (err) {
		console.error('Frame UI Loader Error:', err);
	}
})();
