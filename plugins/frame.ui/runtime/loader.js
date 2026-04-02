// Frame Runtime Loader
// WASM loader with plugin-extensible bridge
// Version: 3.0.0
//
// Architecture:
//   1. Creates __cleanRuntime shared object (synchronous)
//   2. Registers frame.ui bridge functions
//   3. Other plugins (frame.client, etc.) register via __cleanRuntime.registerEnv()
//   4. Loads and instantiates WASM with combined env from all plugins
//
// Plugin scripts must load BEFORE this script (they call registerEnv synchronously).
// This script loads LAST and triggers WASM instantiation.

// --- Phase 1: Shared Runtime (synchronous) ---

(function() {
	let memory, instance;
	let heapPtr = 4096;
	const pendingEnv = {};

	function readString(ptr, len) {
		const bytes = new Uint8Array(memory.buffer, ptr, len);
		return new TextDecoder().decode(bytes);
	}

	function writeString(str) {
		const bytes = new TextEncoder().encode(str);
		const ptr = heapPtr;
		const view = new Uint8Array(memory.buffer, ptr, bytes.length + 8);
		new DataView(memory.buffer).setUint32(ptr, bytes.length, true);
		new DataView(memory.buffer).setUint32(ptr + 4, bytes.length, true);
		view.set(bytes, 8);
		heapPtr += bytes.length + 8 + (8 - (bytes.length % 8));
		return ptr;
	}

	function memAlloc(size) {
		const ptr = heapPtr;
		heapPtr += size + (8 - (size % 8));
		return ptr;
	}

	window.__cleanRuntime = {
		readString: function(ptr, len) { return readString(ptr, len); },
		writeString: function(str) { return writeString(str); },
		memAlloc: function(size) { return memAlloc(size); },
		getInstance: function() { return instance; },
		registerEnv: function(fns) { Object.assign(pendingEnv, fns); },
		_getPendingEnv: function() { return pendingEnv; },
		_setMemoryAndInstance: function(m, i, hp) {
			memory = m;
			instance = i;
			if (hp !== undefined) heapPtr = hp;
		}
	};
})();

// --- Phase 2: frame.ui Bridge Registration ---

(function() {
	const rt = window.__cleanRuntime;

	// --- frame.ui State ---
	let currentEvent = null;
	let currentEventTarget = null;
	const componentRegistry = new Map();
	const slotStore = new Map();
	const stateStore = new Map();
	const registeredEventTypes = new Set();
	const eventHandlers = new Map();

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
						const fn = rt.getInstance().exports['handle_event_' + handlerIdx];
						if (fn) fn();
						currentEvent = null;
						currentEventTarget = null;
					});
				}
			}
		});
	}

	rt.registerEnv({
		// Console
		print: (ptr, len) => console.log(rt.readString(ptr, len)),
		printl: (ptr, len) => console.log(rt.readString(ptr, len)),
		'string.concat': (ptr1, len1, ptr2, len2) => {
			return rt.writeString(rt.readString(ptr1, len1) + rt.readString(ptr2, len2));
		},

		// Memory management
		mem_alloc: (size) => {
			return rt.memAlloc(size);
		},
		mem_retain: (ptr) => {},
		mem_release: (ptr) => {},
		mem_scope_push: () => {},
		mem_scope_pop: () => {},

		// ========== HTML Rendering ==========

		_html_escape: (strPtr, strLen) => {
			const str = rt.readString(strPtr, strLen);
			const escaped = str
				.replace(/&/g, '&amp;')
				.replace(/</g, '&lt;')
				.replace(/>/g, '&gt;')
				.replace(/"/g, '&quot;')
				.replace(/'/g, '&#039;');
			return rt.writeString(escaped);
		},

		_html_raw: (strPtr, strLen) => {
			return rt.writeString(rt.readString(strPtr, strLen));
		},

		// ========== Component Registry ==========

		_ui_registerComponent: (tagPtr, tagLen, classPtr, classLen) => {
			componentRegistry.set(rt.readString(tagPtr, tagLen), rt.readString(classPtr, classLen));
			return 0;
		},

		_ui_getComponent: (tagPtr, tagLen) => {
			return rt.writeString(componentRegistry.get(rt.readString(tagPtr, tagLen)) || '');
		},

		// ========== Layout Loading ==========

		_ui_loadLayout: (namePtr, nameLen) => {
			return rt.writeString('');
		},

		// ========== Slot Management ==========

		_ui_setSlot: (namePtr, nameLen, contentPtr, contentLen) => {
			slotStore.set(rt.readString(namePtr, nameLen), rt.readString(contentPtr, contentLen));
			return 0;
		},

		_ui_getSlot: (namePtr, nameLen) => {
			return rt.writeString(slotStore.get(rt.readString(namePtr, nameLen)) || '');
		},

		// ========== Event Handler Registration ==========

		_ui_onEvent: (selectorPtr, selectorLen, eventTypePtr, eventTypeLen, handlerIdx) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const eventType = rt.readString(eventTypePtr, eventTypeLen);
			const key = selector + '\0' + eventType;

			if (!eventHandlers.has(key)) {
				eventHandlers.set(key, []);
			}
			eventHandlers.get(key).push(handlerIdx);
			ensureDocumentListener(eventType);
			return 0;
		},

		// ========== State Management ==========

		_ui_setState: (idPtr, idLen, jsonPtr, jsonLen) => {
			stateStore.set(rt.readString(idPtr, idLen), rt.readString(jsonPtr, jsonLen));
			return 0;
		},

		_ui_getState: (idPtr, idLen) => {
			return rt.writeString(stateStore.get(rt.readString(idPtr, idLen)) || '{}');
		},

		// ========== DOM Manipulation ==========

		_ui_updateElement: (selectorPtr, selectorLen, contentPtr, contentLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (el) { el.innerHTML = rt.readString(contentPtr, contentLen); return 0; }
			return -1;
		},

		_ui_updateAttr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (el) { el.setAttribute(rt.readString(attrPtr, attrLen), rt.readString(valPtr, valLen)); return 0; }
			return -1;
		},

		// ========== Form Binding ==========

		_ui_bindInput: (selectorPtr, selectorLen, pathPtr, pathLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const path = rt.readString(pathPtr, pathLen);
			const el = document.querySelector(selector);
			if (!el) return -1;

			el.addEventListener('input', () => {
				stateStore.set(path, JSON.stringify(el.value));
				const setterName = 'set_' + path.replace(/\./g, '_');
				if (rt.getInstance().exports[setterName]) {
					rt.getInstance().exports[setterName](rt.writeString(el.value));
				}
			});
			return 0;
		},

		// ========== Validation ==========

		_ui_validate: (valuePtr, valueLen, rulePtr, ruleLen) => {
			const value = rt.readString(valuePtr, valueLen);
			const rule = rt.readString(rulePtr, ruleLen);
			let error = '';
			switch (rule) {
				case 'required': if (!value.trim()) error = 'This field is required'; break;
				case 'email': if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) error = 'Invalid email'; break;
				case 'url': try { new URL(value); } catch { error = 'Invalid URL'; } break;
			}
			return rt.writeString(error);
		},

		// ========== Event Handler Context ==========

		_ui_eventAttr: (attrPtr, attrLen) => {
			if (!currentEventTarget) return rt.writeString('');
			return rt.writeString(currentEventTarget.getAttribute(rt.readString(attrPtr, attrLen)) || '');
		},

		_ui_eventValue: () => {
			if (!currentEventTarget) return rt.writeString('');
			if (currentEventTarget.tagName === 'INPUT' || currentEventTarget.tagName === 'TEXTAREA' || currentEventTarget.tagName === 'SELECT') {
				return rt.writeString(currentEventTarget.value || '');
			}
			return rt.writeString(currentEventTarget.textContent || '');
		},

		_ui_eventClosestAttr: (selectorPtr, selectorLen, attrPtr, attrLen) => {
			if (!currentEventTarget) return rt.writeString('');
			const el = currentEventTarget.closest(rt.readString(selectorPtr, selectorLen));
			if (!el) return rt.writeString('');
			return rt.writeString(el.getAttribute(rt.readString(attrPtr, attrLen)) || '');
		},

		_ui_eventType: () => {
			if (!currentEvent) return rt.writeString('');
			return rt.writeString(currentEvent.type);
		},

		// ========== Clipboard ==========

		_ui_clipboardWrite: (textPtr, textLen) => {
			navigator.clipboard.writeText(rt.readString(textPtr, textLen)).catch(() => {});
			return 0;
		},

		// ========== URL / Location ==========

		_ui_locationHref: (urlPtr, urlLen) => {
			window.location.href = rt.readString(urlPtr, urlLen);
			return 0;
		},

		_ui_locationQuery: (paramPtr, paramLen) => {
			const value = new URLSearchParams(window.location.search).get(rt.readString(paramPtr, paramLen)) || '';
			return rt.writeString(value);
		},

		_ui_locationPath: () => {
			return rt.writeString(window.location.pathname);
		},

		// ========== DOM Query (single element) ==========

		_ui_getText: (selectorPtr, selectorLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			return rt.writeString(el ? (el.textContent || '') : '');
		},

		_ui_getAttr: (selectorPtr, selectorLen, attrPtr, attrLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			return rt.writeString(el ? (el.getAttribute(rt.readString(attrPtr, attrLen)) || '') : '');
		},

		_ui_toggleClass: (selectorPtr, selectorLen, classPtr, classLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (el) el.classList.toggle(rt.readString(classPtr, classLen));
			return 0;
		},

		_ui_addClass: (selectorPtr, selectorLen, classPtr, classLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (el) el.classList.add(rt.readString(classPtr, classLen));
			return 0;
		},

		_ui_removeClass: (selectorPtr, selectorLen, classPtr, classLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (el) el.classList.remove(rt.readString(classPtr, classLen));
			return 0;
		},

		_ui_setStyle: (selectorPtr, selectorLen, propPtr, propLen, valPtr, valLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (el) el.style[rt.readString(propPtr, propLen)] = rt.readString(valPtr, valLen);
			return 0;
		},

		_ui_updateElementSelf: (contentPtr, contentLen) => {
			if (!currentEventTarget) return -1;
			currentEventTarget.textContent = rt.readString(contentPtr, contentLen);
			return 0;
		},

		// ========== DOM Batch (querySelectorAll) ==========

		_ui_querySetStyle: (selectorPtr, selectorLen, propPtr, propLen, valPtr, valLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const prop = rt.readString(propPtr, propLen);
			const val = rt.readString(valPtr, valLen);
			document.querySelectorAll(selector).forEach(el => { el.style[prop] = val; });
			return 0;
		},

		_ui_querySetAttr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const attr = rt.readString(attrPtr, attrLen);
			const val = rt.readString(valPtr, valLen);
			document.querySelectorAll(selector).forEach(el => el.setAttribute(attr, val));
			return 0;
		},

		_ui_queryAddClass: (selectorPtr, selectorLen, classPtr, classLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const cls = rt.readString(classPtr, classLen);
			document.querySelectorAll(selector).forEach(el => el.classList.add(cls));
			return 0;
		},

		_ui_queryRemoveClass: (selectorPtr, selectorLen, classPtr, classLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const cls = rt.readString(classPtr, classLen);
			document.querySelectorAll(selector).forEach(el => el.classList.remove(cls));
			return 0;
		},

		_ui_filterByAttr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const attr = rt.readString(attrPtr, attrLen);
			const val = rt.readString(valPtr, valLen);
			document.querySelectorAll(selector).forEach(el => {
				if (val === '*' || el.getAttribute(attr) === val) {
					el.style.display = '';
				} else {
					el.style.display = 'none';
				}
			});
			return 0;
		},

		_ui_filterByText: (selectorPtr, selectorLen, nameAttrPtr, nameAttrLen, descAttrPtr, descAttrLen, queryPtr, queryLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const nameAttr = rt.readString(nameAttrPtr, nameAttrLen);
			const descAttr = rt.readString(descAttrPtr, descAttrLen);
			const query = rt.readString(queryPtr, queryLen).toLowerCase();
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

		_ui_observeVisible: (selectorPtr, selectorLen, classPtr, classLen) => {
			const selector = rt.readString(selectorPtr, selectorLen);
			const cls = rt.readString(classPtr, classLen);
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

		// ========== Form Helpers ==========

		_ui_inputValue: (selectorPtr, selectorLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (!el) return rt.writeString('');
			return rt.writeString(el.value || '');
		},

		_ui_formJson: (selectorPtr, selectorLen) => {
			const form = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (!form) return rt.writeString('{}');
			const data = {};
			new FormData(form).forEach((v, k) => { data[k] = v; });
			return rt.writeString(JSON.stringify(data));
		},

		_ui_formData: (selectorPtr, selectorLen) => {
			const form = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (!form) return rt.writeString('');
			const params = new URLSearchParams();
			new FormData(form).forEach((v, k) => params.append(k, v));
			return rt.writeString(params.toString());
		},

		_ui_checked: (selectorPtr, selectorLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			return (el && el.checked) ? 1 : 0;
		},

		_ui_setInput: (selectorPtr, selectorLen, valPtr, valLen) => {
			const el = document.querySelector(rt.readString(selectorPtr, selectorLen));
			if (!el) return -1;
			el.value = rt.readString(valPtr, valLen);
			return 0;
		},

		// ========== Timers ==========

		_ui_setTimeout: (handlerIdx, delayMs) => {
			setTimeout(() => {
				const fn = rt.getInstance().exports['handle_event_' + handlerIdx];
				if (fn) {
					fn();
				}
			}, delayMs);
			return 0;
		},

		// ========== Theming ==========

		_ui_injectHeadCss: (cssPtr, cssLen) => {
			const style = document.createElement('style');
			style.textContent = rt.readString(cssPtr, cssLen);
			document.head.appendChild(style);
			return 0;
		},
	});
})();

// --- Phase 3: WASM Loading (async, runs after all plugins registered) ---

(async function() {
	const rt = window.__cleanRuntime;
	const script = document.querySelector('script[data-wasm]');
	const wasmPath = (script && script.dataset.wasm) || 'frontend.wasm';

	try {
		const response = await fetch(wasmPath);
		const bytes = await response.arrayBuffer();
		const bridge = { env: rt._getPendingEnv() };
		const module = await WebAssembly.instantiate(bytes, bridge);
		const instance = module.instance;
		const memory = instance.exports.memory;

		let heapPtr = 4096;
		if (instance.exports.__heap_ptr) {
			heapPtr = instance.exports.__heap_ptr.value;
		}

		rt._setMemoryAndInstance(memory, instance, heapPtr);

		if (instance.exports._start) {
			instance.exports._start();
		} else if (instance.exports.start) {
			instance.exports.start();
		}

	} catch (err) {
		console.error('Frame Runtime Loader Error:', err);
	}
})();
