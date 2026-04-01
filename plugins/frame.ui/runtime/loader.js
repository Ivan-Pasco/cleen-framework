// Frame UI Runtime Loader
// WASM loader with full browser API bridge
// Version: 2.2.0
//
// All interactivity uses _ui_onEvent delegation + targeted DOM updates.
// WASM _start() registers handlers, handlers update DOM via bridge functions.

(async function() {
	const script = document.currentScript;
	const wasmPath = script.dataset.wasm || 'frontend.wasm';

	// --- State ---
	let memory, instance;
	let heapPtr = 4096;
	let currentEvent = null;
	let currentEventTarget = null;
	const componentRegistry = new Map();
	const slotStore = new Map();
	const stateStore = new Map();
	const registeredEventTypes = new Set();
	const eventHandlers = new Map(); // "selector\0eventType" -> [handlerIdx, ...]

	// --- Memory Helpers ---

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
		env: {
			// Console
			print: (ptr, len) => console.log(readString(ptr, len)),
			printl: (ptr, len) => console.log(readString(ptr, len)),
			'string.concat': (ptr1, len1, ptr2, len2) => {
				return writeString(readString(ptr1, len1) + readString(ptr2, len2));
			},

			// Memory management
			mem_alloc: (size) => {
				const ptr = heapPtr;
				heapPtr += size + (8 - (size % 8));
				return ptr;
			},
			mem_retain: (ptr) => {},
			mem_release: (ptr) => {},
			mem_scope_push: () => {},
			mem_scope_pop: () => {},

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

			_ui_registerComponent: (tagPtr, tagLen, classPtr, classLen) => {
				componentRegistry.set(readString(tagPtr, tagLen), readString(classPtr, classLen));
				return 0;
			},

			_ui_getComponent: (tagPtr, tagLen) => {
				return writeString(componentRegistry.get(readString(tagPtr, tagLen)) || '');
			},

			// ========== Slot Management ==========

			_ui_setSlot: (namePtr, nameLen, contentPtr, contentLen) => {
				slotStore.set(readString(namePtr, nameLen), readString(contentPtr, contentLen));
				return 0;
			},

			_ui_getSlot: (namePtr, nameLen) => {
				return writeString(slotStore.get(readString(namePtr, nameLen)) || '');
			},

			// ========== Event Handler Registration ==========

			_ui_onEvent: (selectorPtr, selectorLen, eventTypePtr, eventTypeLen, handlerIdx) => {
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

			_ui_setState: (idPtr, idLen, jsonPtr, jsonLen) => {
				stateStore.set(readString(idPtr, idLen), readString(jsonPtr, jsonLen));
				return 0;
			},

			_ui_getState: (idPtr, idLen) => {
				return writeString(stateStore.get(readString(idPtr, idLen)) || '{}');
			},

			// ========== DOM Manipulation ==========

			_ui_updateElement: (selectorPtr, selectorLen, contentPtr, contentLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) { el.innerHTML = readString(contentPtr, contentLen); return 0; }
				return -1;
			},

			_ui_updateAttr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) { el.setAttribute(readString(attrPtr, attrLen), readString(valPtr, valLen)); return 0; }
				return -1;
			},

			// ========== Form Binding ==========

			_ui_bindInput: (selectorPtr, selectorLen, pathPtr, pathLen) => {
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

			_ui_eventAttr: (attrPtr, attrLen) => {
				if (!currentEventTarget) return writeString('');
				return writeString(currentEventTarget.getAttribute(readString(attrPtr, attrLen)) || '');
			},

			_ui_eventValue: () => {
				if (!currentEventTarget) return writeString('');
				if (currentEventTarget.tagName === 'INPUT' || currentEventTarget.tagName === 'TEXTAREA' || currentEventTarget.tagName === 'SELECT') {
					return writeString(currentEventTarget.value || '');
				}
				return writeString(currentEventTarget.textContent || '');
			},

			_ui_eventClosestAttr: (selectorPtr, selectorLen, attrPtr, attrLen) => {
				if (!currentEventTarget) return writeString('');
				const el = currentEventTarget.closest(readString(selectorPtr, selectorLen));
				if (!el) return writeString('');
				return writeString(el.getAttribute(readString(attrPtr, attrLen)) || '');
			},

			_ui_eventType: () => {
				if (!currentEvent) return writeString('');
				return writeString(currentEvent.type);
			},

			// ========== Clipboard ==========

			_ui_clipboardWrite: (textPtr, textLen) => {
				navigator.clipboard.writeText(readString(textPtr, textLen)).catch(() => {});
				return 0;
			},

			// ========== URL / Location ==========

			_ui_locationHref: (urlPtr, urlLen) => {
				window.location.href = readString(urlPtr, urlLen);
				return 0;
			},

			_ui_locationQuery: (paramPtr, paramLen) => {
				const value = new URLSearchParams(window.location.search).get(readString(paramPtr, paramLen)) || '';
				return writeString(value);
			},

			_ui_locationPath: () => {
				return writeString(window.location.pathname);
			},

			// ========== DOM Query (single element) ==========

			_ui_getText: (selectorPtr, selectorLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				return writeString(el ? (el.textContent || '') : '');
			},

			_ui_getAttr: (selectorPtr, selectorLen, attrPtr, attrLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				return writeString(el ? (el.getAttribute(readString(attrPtr, attrLen)) || '') : '');
			},

			_ui_toggleClass: (selectorPtr, selectorLen, classPtr, classLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.classList.toggle(readString(classPtr, classLen));
				return 0;
			},

			_ui_addClass: (selectorPtr, selectorLen, classPtr, classLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.classList.add(readString(classPtr, classLen));
				return 0;
			},

			_ui_removeClass: (selectorPtr, selectorLen, classPtr, classLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.classList.remove(readString(classPtr, classLen));
				return 0;
			},

			_ui_setStyle: (selectorPtr, selectorLen, propPtr, propLen, valPtr, valLen) => {
				const el = document.querySelector(readString(selectorPtr, selectorLen));
				if (el) el.style[readString(propPtr, propLen)] = readString(valPtr, valLen);
				return 0;
			},

			_ui_updateElementSelf: (contentPtr, contentLen) => {
				if (!currentEventTarget) return -1;
				currentEventTarget.textContent = readString(contentPtr, contentLen);
				return 0;
			},

			// ========== DOM Batch (querySelectorAll) ==========

			_ui_querySetStyle: (selectorPtr, selectorLen, propPtr, propLen, valPtr, valLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const prop = readString(propPtr, propLen);
				const val = readString(valPtr, valLen);
				document.querySelectorAll(selector).forEach(el => { el.style[prop] = val; });
				return 0;
			},

			_ui_querySetAttr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const attr = readString(attrPtr, attrLen);
				const val = readString(valPtr, valLen);
				document.querySelectorAll(selector).forEach(el => el.setAttribute(attr, val));
				return 0;
			},

			_ui_queryAddClass: (selectorPtr, selectorLen, classPtr, classLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const cls = readString(classPtr, classLen);
				document.querySelectorAll(selector).forEach(el => el.classList.add(cls));
				return 0;
			},

			_ui_queryRemoveClass: (selectorPtr, selectorLen, classPtr, classLen) => {
				const selector = readString(selectorPtr, selectorLen);
				const cls = readString(classPtr, classLen);
				document.querySelectorAll(selector).forEach(el => el.classList.remove(cls));
				return 0;
			},

			_ui_filterByAttr: (selectorPtr, selectorLen, attrPtr, attrLen, valPtr, valLen) => {
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

			_ui_filterByText: (selectorPtr, selectorLen, nameAttrPtr, nameAttrLen, descAttrPtr, descAttrLen, queryPtr, queryLen) => {
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

			_ui_observeVisible: (selectorPtr, selectorLen, classPtr, classLen) => {
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

			_ui_setTimeout: (handlerIdx, delayMs) => {
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
		}
	};

	// --- WASM Loading ---

	try {
		const response = await fetch(wasmPath);
		const bytes = await response.arrayBuffer();
		const module = await WebAssembly.instantiate(bytes, bridge);
		instance = module.instance;
		memory = instance.exports.memory;

		if (instance.exports.__heap_ptr) {
			heapPtr = instance.exports.__heap_ptr.value;
		}

		// Call _start — registers event handlers via _ui_onEvent
		if (instance.exports._start) {
			instance.exports._start();
		} else if (instance.exports.start) {
			instance.exports.start();
		}

	} catch (err) {
		console.error('Frame UI Loader Error:', err);
	}
})();
