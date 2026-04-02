// Frame Client Bridge
// Registers api.*, live.*, feed.* bridge functions into __cleanRuntime
// Version: 1.0.0
//
// This file must load BEFORE loader.js (which instantiates WASM).
// It does NOT load WASM -- it only registers bridge functions.

(function() {
	const rt = window.__cleanRuntime;
	if (!rt) {
		console.error('frame.client: __cleanRuntime not found. Ensure loader.js defines it before bridge.js runs.');
		return;
	}

	// ========== API: HTTP State ==========

	let pendingHeaders = {};
	let pendingTimeout = 0;
	let storedAuth = null;

	let currentResponse = null;
	let currentResponseBody = null;
	let currentResponseHeaders = null;

	function dispatchResponse(resp, body, handlerIdx) {
		const headers = {};
		resp.headers.forEach((v, k) => { headers[k] = v; });
		currentResponse = { status: resp.status, ok: resp.ok };
		currentResponseBody = body;
		currentResponseHeaders = headers;
		const fn = rt.getInstance().exports['handle_event_' + handlerIdx];
		if (fn) fn();
		currentResponse = null;
		currentResponseBody = null;
		currentResponseHeaders = null;
	}

	function dispatchError(err, handlerIdx) {
		currentResponse = { status: 0, ok: false };
		currentResponseBody = JSON.stringify({ error: err.message || String(err) });
		currentResponseHeaders = {};
		const fn = rt.getInstance().exports['handle_event_' + handlerIdx];
		if (fn) fn();
		currentResponse = null;
		currentResponseBody = null;
		currentResponseHeaders = null;
	}

	function buildOptions(method, body) {
		const headers = { ...pendingHeaders };
		const timeout = pendingTimeout;
		pendingHeaders = {};
		pendingTimeout = 0;

		if (storedAuth && !headers['Authorization']) {
			if (storedAuth.scheme === 'bearer') {
				headers['Authorization'] = 'Bearer ' + storedAuth.credential;
			} else if (storedAuth.scheme === 'basic') {
				headers['Authorization'] = 'Basic ' + storedAuth.credential;
			}
		}

		const options = { method, headers };
		if (body && method !== 'GET' && method !== 'HEAD') {
			options.body = body;
			if (!headers['Content-Type']) {
				headers['Content-Type'] = 'application/json';
			}
		}
		if (!headers['Accept']) {
			headers['Accept'] = 'application/json';
		}

		if (timeout > 0) {
			const controller = new AbortController();
			options.signal = controller.signal;
			setTimeout(() => controller.abort(), timeout);
		}

		return options;
	}

	function doFetch(url, method, body, handlerIdx) {
		const options = buildOptions(method, body);
		fetch(url, options)
			.then(async (resp) => dispatchResponse(resp, await resp.text(), handlerIdx))
			.catch((err) => dispatchError(err, handlerIdx));
		return 0;
	}

	// ========== Live: WebSocket State ==========

	let liveNextId = 1;
	const liveConnections = new Map();
	let currentLiveMessage = null;
	let currentLiveConnId = 0;
	let currentLiveCloseCode = 0;
	let currentLiveCloseReason = null;
	let currentLiveError = null;

	// ========== Feed: SSE State ==========

	let feedNextId = 1;
	const feedConnections = new Map();
	let currentFeedData = null;
	let currentFeedEventType = null;
	let currentFeedLastId = null;
	let currentFeedConnId = 0;

	// ========== Register All Bridge Functions ==========

	rt.registerEnv({

		// ========== API: HTTP Requests ==========

		_api_get: (urlPtr, urlLen, handlerIdx) => {
			return doFetch(rt.readString(urlPtr, urlLen), 'GET', null, handlerIdx);
		},

		_api_post: (urlPtr, urlLen, bodyPtr, bodyLen, handlerIdx) => {
			return doFetch(rt.readString(urlPtr, urlLen), 'POST', rt.readString(bodyPtr, bodyLen), handlerIdx);
		},

		_api_put: (urlPtr, urlLen, bodyPtr, bodyLen, handlerIdx) => {
			return doFetch(rt.readString(urlPtr, urlLen), 'PUT', rt.readString(bodyPtr, bodyLen), handlerIdx);
		},

		_api_patch: (urlPtr, urlLen, bodyPtr, bodyLen, handlerIdx) => {
			return doFetch(rt.readString(urlPtr, urlLen), 'PATCH', rt.readString(bodyPtr, bodyLen), handlerIdx);
		},

		_api_delete: (urlPtr, urlLen, handlerIdx) => {
			return doFetch(rt.readString(urlPtr, urlLen), 'DELETE', null, handlerIdx);
		},

		_api_header: (namePtr, nameLen, valPtr, valLen) => {
			pendingHeaders[rt.readString(namePtr, nameLen)] = rt.readString(valPtr, valLen);
			return 0;
		},

		_api_timeout: (ms) => {
			pendingTimeout = ms;
			return 0;
		},

		_api_auth: (schemePtr, schemeLen, credPtr, credLen) => {
			storedAuth = {
				scheme: rt.readString(schemePtr, schemeLen),
				credential: rt.readString(credPtr, credLen)
			};
			return 0;
		},

		_api_clearAuth: () => {
			storedAuth = null;
			return 0;
		},

		_api_submit: (formPtr, formLen, urlPtr, urlLen, methodPtr, methodLen, handlerIdx) => {
			const form = document.querySelector(rt.readString(formPtr, formLen));
			if (!form) {
				dispatchError(new Error('Form not found'), handlerIdx);
				return -1;
			}
			const data = {};
			new FormData(form).forEach((v, k) => { data[k] = v; });
			return doFetch(
				rt.readString(urlPtr, urlLen),
				rt.readString(methodPtr, methodLen).toUpperCase(),
				JSON.stringify(data),
				handlerIdx
			);
		},

		// ========== API: Response Context ==========

		_api_status: () => currentResponse ? currentResponse.status : 0,

		_api_body: () => rt.writeString(currentResponseBody || ''),

		_api_ok: () => (currentResponse && currentResponse.ok) ? 1 : 0,

		_api_json: (pathPtr, pathLen) => {
			if (!currentResponseBody) return rt.writeString('');
			const path = rt.readString(pathPtr, pathLen);
			try {
				let obj = JSON.parse(currentResponseBody);
				if (path === '') return rt.writeString(currentResponseBody);
				for (const part of path.split('.')) {
					if (obj == null) return rt.writeString('');
					obj = obj[part];
				}
				if (obj == null) return rt.writeString('');
				return rt.writeString(typeof obj === 'object' ? JSON.stringify(obj) : String(obj));
			} catch {
				return rt.writeString('');
			}
		},

		_api_responseHeader: (namePtr, nameLen) => {
			if (!currentResponseHeaders) return rt.writeString('');
			const name = rt.readString(namePtr, nameLen).toLowerCase();
			return rt.writeString(currentResponseHeaders[name] || '');
		},

		// ========== Live: WebSocket ==========

		_live_open: (urlPtr, urlLen, onMsgIdx, onCloseIdx, onErrorIdx) => {
			const url = rt.readString(urlPtr, urlLen);
			const connId = liveNextId++;

			try {
				const ws = new WebSocket(url);
				liveConnections.set(connId, ws);

				ws.addEventListener('message', (e) => {
					currentLiveMessage = typeof e.data === 'string' ? e.data : '';
					currentLiveConnId = connId;
					const fn = rt.getInstance().exports['handle_event_' + onMsgIdx];
					if (fn) fn();
					currentLiveMessage = null;
					currentLiveConnId = 0;
				});

				ws.addEventListener('close', (e) => {
					currentLiveConnId = connId;
					currentLiveCloseCode = e.code;
					currentLiveCloseReason = e.reason || '';
					const fn = rt.getInstance().exports['handle_event_' + onCloseIdx];
					if (fn) fn();
					currentLiveConnId = 0;
					currentLiveCloseCode = 0;
					currentLiveCloseReason = null;
					liveConnections.delete(connId);
				});

				ws.addEventListener('error', () => {
					currentLiveConnId = connId;
					currentLiveError = 'WebSocket error';
					const fn = rt.getInstance().exports['handle_event_' + onErrorIdx];
					if (fn) fn();
					currentLiveConnId = 0;
					currentLiveError = null;
				});
			} catch (err) {
				currentLiveConnId = connId;
				currentLiveError = err.message;
				const fn = rt.getInstance().exports['handle_event_' + onErrorIdx];
				if (fn) fn();
				currentLiveConnId = 0;
				currentLiveError = null;
				return 0;
			}

			return connId;
		},

		_live_send: (connId, msgPtr, msgLen) => {
			const ws = liveConnections.get(connId);
			if (!ws || ws.readyState !== WebSocket.OPEN) return -1;
			ws.send(rt.readString(msgPtr, msgLen));
			return 0;
		},

		_live_close: (connId) => {
			const ws = liveConnections.get(connId);
			if (ws) ws.close();
			return 0;
		},

		_live_state: (connId) => {
			const ws = liveConnections.get(connId);
			if (!ws) return rt.writeString('closed');
			return rt.writeString(['connecting', 'open', 'closing', 'closed'][ws.readyState] || 'closed');
		},

		// Live Context
		_live_message: () => rt.writeString(currentLiveMessage || ''),
		_live_connId: () => currentLiveConnId,
		_live_closeCode: () => currentLiveCloseCode,
		_live_closeReason: () => rt.writeString(currentLiveCloseReason || ''),
		_live_error: () => rt.writeString(currentLiveError || ''),

		// ========== Feed: Server-Sent Events ==========

		_feed_open: (urlPtr, urlLen, onMsgIdx, onErrorIdx) => {
			const url = rt.readString(urlPtr, urlLen);
			const connId = feedNextId++;

			try {
				const source = new EventSource(url);
				feedConnections.set(connId, source);

				source.addEventListener('message', (e) => {
					currentFeedData = e.data || '';
					currentFeedEventType = 'message';
					currentFeedLastId = e.lastEventId || '';
					currentFeedConnId = connId;
					const fn = rt.getInstance().exports['handle_event_' + onMsgIdx];
					if (fn) fn();
					currentFeedData = null;
					currentFeedEventType = null;
					currentFeedLastId = null;
					currentFeedConnId = 0;
				});

				source.addEventListener('error', () => {
					currentFeedConnId = connId;
					currentFeedData = 'SSE connection error';
					const fn = rt.getInstance().exports['handle_event_' + onErrorIdx];
					if (fn) fn();
					currentFeedConnId = 0;
					currentFeedData = null;
				});
			} catch {
				return 0;
			}

			return connId;
		},

		_feed_on: (connId, eventPtr, eventLen, handlerIdx) => {
			const source = feedConnections.get(connId);
			if (!source) return -1;
			const eventName = rt.readString(eventPtr, eventLen);

			source.addEventListener(eventName, (e) => {
				currentFeedData = e.data || '';
				currentFeedEventType = eventName;
				currentFeedLastId = e.lastEventId || '';
				currentFeedConnId = connId;
				const fn = rt.getInstance().exports['handle_event_' + handlerIdx];
				if (fn) fn();
				currentFeedData = null;
				currentFeedEventType = null;
				currentFeedLastId = null;
				currentFeedConnId = 0;
			});

			return 0;
		},

		_feed_close: (connId) => {
			const source = feedConnections.get(connId);
			if (source) {
				source.close();
				feedConnections.delete(connId);
			}
			return 0;
		},

		// Feed Context
		_feed_data: () => rt.writeString(currentFeedData || ''),
		_feed_eventType: () => rt.writeString(currentFeedEventType || ''),
		_feed_lastId: () => rt.writeString(currentFeedLastId || ''),
		_feed_connId: () => currentFeedConnId,
	});
})();
