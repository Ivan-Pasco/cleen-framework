# Frame Client Communication Specification (14)

**Project:** Frame -- Full-Stack Framework for Clean Language
**Version:** 2.0
**Status:** Specification
**Location:** `/documents/specification/14_frame_ui_client_communication.md`

---

## 1. Overview

This specification defines **frame.client** -- the plugin for all client-side communication between the browser and the server. It provides three namespaces:

| Namespace | Protocol | What it does |
|-----------|----------|-------------|
| `api.*` | HTTP | Call APIs, submit forms, read responses |
| `live.*` | WebSocket | Bidirectional real-time connections |
| `feed.*` | SSE | Server-push event streams |

It also specifies form-reading helpers added to **frame.ui**.

---

## 2. Architecture

### 2.1 Plugin Map

```
frame.ui      → What you SEE       (DOM, events, components, state, timers)
frame.client  → What you SEND      (api, live, feed -- all client-server communication)
frame.server  → What you SERVE     (endpoints, request context, response helpers)
frame.data    → What you STORE     (models, queries, migrations)
frame.auth    → What you SECURE    (login, sessions, roles)
frame.canvas  → What you DRAW      (graphics, animation, sprites)
```

### 2.2 Why frame.client

| Principle | Decision |
|-----------|----------|
| Plugin name = what it does | `client` = "I'm the client talking to the server" |
| No namespace collision | Server uses `http.*` (sync). Client uses `api.*` (async). No overlap. |
| One import, three protocols | `import frame.client` gives you `api.*`, `live.*`, `feed.*` |
| Names a beginner understands | "API", "live connection", "feed" -- zero jargon |

### 2.3 Naming Rationale

| Name | Why this word |
|------|--------------|
| `api` | Every developer says "call the API." Most natural word for HTTP requests. 3 characters. |
| `live` | Everyone understands "live connection." Like "live chat." No networking knowledge required. |
| `feed` | Everyone knows "news feed." Data flows to you. Intuitive for server-push. |

### 2.4 Server vs Client -- No Confusion

```clean
// Server-side (frame.server): synchronous, returns directly
string data = http.get("https://external-api.com/data")

// Client-side (frame.client): asynchronous, dispatches to handler by name
integer s = api.get("/api/users", "onUsersLoaded")
```

Different plugins. Different namespaces. Different calling conventions. Clear.

### 2.5 Rename: frame.server → frame.server

| Before | After | Why |
|--------|-------|-----|
| `import frame.server` | `import frame.server` | Shorter, simpler, says exactly what it is |

Developer-facing syntax is unchanged: `endpoints:`, `req.*`, `res.*`, `json()`, `redirect()`.

---

## 3. Async Model: Named Handlers

All client communication dispatches to a **named handler function** passed as a string:

```clean
start:
    integer s = ui.onEvent("#btn", "click", "loadUsers")

functions:
    loadUsers()
        integer s = api.get("/api/users", "onUsersLoaded")

    onUsersLoaded()
        string name = api.json("name")
        integer s = ui.updateElement("#user-name", name)
```

1. A function initiates an async operation with a **handler function name** (string literal) as parameter
2. When the operation completes, the runtime calls `instance.exports[handlerName]`
3. Context functions provide data inside the handler (`api.json()`, `live.message()`, etc.)

### 3.1 Handler Parameters — String Names

Plugin bridge functions declare handler parameters as `"string"` with `expand_strings = true`:

```toml
{ name = "_api_get", params = ["string", "string"], returns = "integer", expand_strings = true, ... }
```

The runtime:
1. Reads the handler-name string from WASM memory (pointer + length pair)
2. Looks up `instance.exports[handlerName]` on the active module
3. Calls it with no arguments (context is read via `api.*`, `live.*`, `feed.*` accessors)
4. Logs `console.error` to the browser console if the name has no matching export — mistyped names surface immediately instead of being silently dropped

Because top-level Clean functions are exported by default, any function declared in the module's `functions:` block is a valid handler. Handlers must be **top-level** functions — methods on classes are not callable through the export table.

This replaces an earlier design that attempted to pass function references as indices into a WASM indirect function table. That design depended on compiler support for function pointers (`COMPILER_FN_POINTERS_UNIMPL`) and silently dropped every registration when the table was absent. The current design works with the compiler as shipped today.

### 3.2 Why Not Synchronous

SharedArrayBuffer + Atomics could block WASM on async. Rejected because:
- Requires COOP/COEP headers (breaks iframes, CDNs, embeds)
- Needs a Web Worker (bundle size, complexity)
- Hidden blocking is surprising
- Incompatible with some mobile/WebView environments

---

## 4. Form Helpers (frame.ui additions)

Reading DOM inputs is a UI operation. These belong in frame.ui.

### 4.1 Functions

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `ui.inputValue(selector)` | `_ui_inputValue` | `string` | `string` | Get value of input/textarea/select |
| `ui.formJson(selector)` | `_ui_formJson` | `string` | `string` | Collect all named form inputs as JSON string |
| `ui.formData(selector)` | `_ui_formData` | `string` | `string` | Collect as URL-encoded key=value string |
| `ui.checked(selector)` | `_ui_checked` | `string` | `integer` | Checkbox checked state (0/1) |
| `ui.setInput(selector, value)` | `_ui_setInput` | `string, string` | `integer` | Set input/textarea/select value |

### 4.2 plugin.toml additions (frame.ui)

```toml
# Form Helpers
{ name = "_ui_inputValue", params = ["string"], returns = "string", description = "Get value of input/textarea/select matching selector", expand_strings = true },
{ name = "_ui_formJson", params = ["string"], returns = "string", description = "Collect all named inputs in form as JSON object string", expand_strings = true },
{ name = "_ui_formData", params = ["string"], returns = "string", description = "Collect all named inputs as URL-encoded key=value string", expand_strings = true },
{ name = "_ui_checked", params = ["string"], returns = "integer", description = "Get checkbox checked state (0=unchecked, 1=checked)", expand_strings = true },
{ name = "_ui_setInput", params = ["string", "string"], returns = "integer", description = "Set value of input/textarea/select", expand_strings = true },
```

### 4.3 loader.js additions (frame.ui runtime)

```javascript
_ui_inputValue: (selectorPtr, selectorLen) => {
    const el = document.querySelector(readString(selectorPtr, selectorLen));
    if (!el) return writeString('');
    return writeString(el.value || '');
},

_ui_formJson: (selectorPtr, selectorLen) => {
    const form = document.querySelector(readString(selectorPtr, selectorLen));
    if (!form) return writeString('{}');
    const data = {};
    new FormData(form).forEach((v, k) => { data[k] = v; });
    return writeString(JSON.stringify(data));
},

_ui_formData: (selectorPtr, selectorLen) => {
    const form = document.querySelector(readString(selectorPtr, selectorLen));
    if (!form) return writeString('');
    const params = new URLSearchParams();
    new FormData(form).forEach((v, k) => params.append(k, v));
    return writeString(params.toString());
},

_ui_checked: (selectorPtr, selectorLen) => {
    const el = document.querySelector(readString(selectorPtr, selectorLen));
    return (el && el.checked) ? 1 : 0;
},

_ui_setInput: (selectorPtr, selectorLen, valPtr, valLen) => {
    const el = document.querySelector(readString(selectorPtr, selectorLen));
    if (!el) return -1;
    el.value = readString(valPtr, valLen);
    return 0;
},
```

---

## 5. API -- HTTP Requests (`api.*`)

### 5.1 Request Functions

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `api.get(url, handlerName)` | `_api_get` | `string, string` | `integer` | GET request |
| `api.post(url, body, handlerName)` | `_api_post` | `string, string, string` | `integer` | POST request |
| `api.put(url, body, handlerName)` | `_api_put` | `string, string, string` | `integer` | PUT request |
| `api.patch(url, body, handlerName)` | `_api_patch` | `string, string, string` | `integer` | PATCH request |
| `api.delete(url, handlerName)` | `_api_delete` | `string, string` | `integer` | DELETE request |
| `api.header(name, value)` | `_api_header` | `string, string` | `integer` | Set header for next request |
| `api.timeout(ms)` | `_api_timeout` | `integer` | `integer` | Set timeout for next request |
| `api.auth(scheme, credential)` | `_api_auth` | `string, string` | `integer` | Set auth for all requests |
| `api.clearAuth()` | `_api_clearAuth` | none | `integer` | Clear stored auth |
| `api.submit(formSelector, url, method, handlerName)` | `_api_submit` | `string, string, string, string` | `integer` | Collect form JSON + send |

### 5.2 Response Context (inside handler)

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `api.status()` | `_api_status` | none | `integer` | HTTP status code (0 = network error) |
| `api.body()` | `_api_body` | none | `string` | Response body as string |
| `api.ok()` | `_api_ok` | none | `integer` | 1 if status 200-299, 0 otherwise |
| `api.json(path)` | `_api_json` | `string` | `string` | Extract value from JSON via dot-notation |
| `api.responseHeader(name)` | `_api_responseHeader` | `string` | `string` | Get response header value |

### 5.3 Error Handling

| Scenario | `api.status()` | `api.ok()` | `api.body()` |
|----------|----------------|------------|--------------|
| Success (200) | `200` | `1` | Response body |
| Not found (404) | `404` | `0` | Error body from server |
| Server error (500) | `500` | `0` | Error body from server |
| Network failure | `0` | `0` | `{"error": "Failed to fetch"}` |
| Timeout | `0` | `0` | `{"error": "The operation was aborted."}` |
| CORS blocked | `0` | `0` | `{"error": "...CORS..."}` |

The response handler is ALWAYS called. No separate error handler. Check `api.ok()` or `api.status()`.

### 5.4 Auth Persistence

```clean
// Set once -- applies to all subsequent api.* calls
api.auth("bearer", token)

// These all include Authorization header automatically:
api.get("/protected/data", "onData")
api.post("/protected/create", body, "onCreated")
api.submit("#form", "/protected/save", "POST", "onSaved")

// Clear when done
api.clearAuth()
```

Manual override for one request: `api.header("Authorization", "Bearer " + token)`.

### 5.5 `api.submit()` -- Convenience

Collects form fields as JSON, sets Content-Type, sends, and dispatches response:

```clean
// Long way:
createUser()
    string body = ui.formJson("#create-form")
    integer s = api.header("Content-Type", "application/json")
    s = api.post("/api/users", body, "onCreated")

// Short way:
createUser()
    integer s = api.submit("#create-form", "/api/users", "POST", "onCreated")
```

---

## 6. Live -- WebSocket (`live.*`)

### 6.1 Connection Functions

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `live.open(url, onMsgName, onCloseName, onErrorName)` | `_live_open` | `string, string, string, string` | `integer` | Open connection. Returns connection ID. |
| `live.send(connId, message)` | `_live_send` | `integer, string` | `integer` | Send message |
| `live.close(connId)` | `_live_close` | `integer` | `integer` | Close connection |
| `live.state(connId)` | `_live_state` | `integer` | `string` | "connecting", "open", "closing", "closed" |

### 6.2 Context Functions (inside handlers)

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `live.message()` | `_live_message` | none | `string` | Message data (in onMessage) |
| `live.connId()` | `_live_connId` | none | `integer` | Connection ID |
| `live.closeCode()` | `_live_closeCode` | none | `integer` | Close code (in onClose) |
| `live.closeReason()` | `_live_closeReason` | none | `string` | Close reason (in onClose) |
| `live.error()` | `_live_error` | none | `string` | Error description (in onError) |

### 6.3 Connection Lifecycle

```
live.open() called
    │
    ▼
[CONNECTING] ─── error ──→ onError handler
    │                       connection removed
    ▼
[OPEN] ─── message ──→ onMessage handler (repeats)
    │
    │ ─── live.close() or server closes
    ▼
[CLOSED] ─── onClose handler
              connection removed
```

---

## 7. Feed -- Server-Sent Events (`feed.*`)

### 7.1 Connection Functions

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `feed.open(url, onMsgName, onErrorName)` | `_feed_open` | `string, string, string` | `integer` | Open feed. Returns connection ID. |
| `feed.on(connId, eventName, handlerName)` | `_feed_on` | `integer, string, string` | `integer` | Listen for named event type |
| `feed.close(connId)` | `_feed_close` | `integer` | `integer` | Close feed |

### 7.2 Context Functions (inside handlers)

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `feed.data()` | `_feed_data` | none | `string` | Event data |
| `feed.eventType()` | `_feed_eventType` | none | `string` | Event type name |
| `feed.lastId()` | `_feed_lastId` | none | `string` | Last event ID |
| `feed.connId()` | `_feed_connId` | none | `integer` | Connection ID |

### 7.3 Named Events

```
Server sends:                    Handler called:
event: message                   → onMsg from feed.open()
data: hello

event: progress                  → handler from feed.on(conn, "progress", N)
data: {"percent": 50}

event: notification              → handler from feed.on(conn, "notification", N)
data: {"msg": "Alert"}
```

### 7.4 Auto-Reconnect

The browser's EventSource auto-reconnects on connection loss. The onError handler fires on each attempt so the developer can update UI status. No configuration needed.

---

## 8. Complete Examples

### 8.1 Simple API Call

```clean
import frame.ui
import frame.client

start:
    integer s = ui.onEvent("#load-btn", "click", "loadUser")

functions:
    loadUser()
        integer s = api.get("/api/users", "onUserLoaded")

    onUserLoaded()
        if api.ok() == 1
            string name = api.json("name")
            integer s = ui.updateElement("#user-name", name)
        else
            integer s = ui.updateElement("#error", "Failed to load")
```

### 8.2 Authenticated CRUD

```clean
import frame.ui
import frame.client

start:
    string token = ui.locationQuery("token")
    integer s = api.auth("bearer", token)
    s = api.get("/api/users", "onUsersLoaded")
    s = ui.onEvent(".delete-btn", "click", "deleteUser")
    s = ui.onEvent("#create-form", "submit", "createUser")

functions:
    onUsersLoaded()
        if api.ok() == 1
            string html = api.body()
            integer s = ui.updateElement("#users-list", html)

    deleteUser()
        string id = ui.eventAttr("data-id")
        integer s = api.delete("/api/users/" + id, "onDeleted")

    onDeleted()
        if api.ok() == 1
            integer s = api.get("/api/users", "onUsersLoaded")

    createUser()
        integer s = api.submit("#create-form", "/api/users", "POST", "onCreated")

    onCreated()
        if api.status() == 201
            integer s = api.get("/api/users", "onUsersLoaded")
            s = ui.setInput("#name", "")
            s = ui.setInput("#email", "")
        else
            string err = api.json("message")
            integer s = ui.updateElement("#form-error", err)
```

### 8.3 Custom Headers

```clean
import frame.ui
import frame.client

start:
    integer s = ui.onEvent("#send-btn", "click", "sendLegacy")

functions:
    sendLegacy()
        integer s = api.header("X-Api-Key", "secret-123")
        s = api.header("Content-Type", "text/xml")
        s = api.timeout(5000)
        s = api.post("/api/legacy", "<xml>data</xml>", "onLegacyResponse")

    onLegacyResponse()
        integer status = api.status()
        string ct = api.responseHeader("content-type")
        string body = api.body()
        integer s = ui.updateElement("#result", body)
```

### 8.4 Live Chat

```clean
import frame.ui
import frame.client

start:
    integer conn = live.open("wss://chat.example.com/room", "onChatMessage", "onChatClosed", "onChatError")
    integer s = ui.setState("conn", conn.toString())
    s = ui.onEvent("#send-btn", "click", "sendMessage")

functions:
    onChatMessage()
        string data = live.message()
        string user = json.get(data, "user")
        string msg = json.get(data, "message")
        string current = ui.getText("#messages")
        integer s = ui.updateElement("#messages", current + "<div><b>" + user + ":</b> " + msg + "</div>")

    onChatClosed()
        integer code = live.closeCode()
        integer s = ui.addClass("#status", "offline")
        s = ui.updateElement("#status", "Disconnected")
        s = ui.setTimeout("reconnectChat", 5000)

    onChatError()
        string err = live.error()
        integer s = ui.updateElement("#status", "Error: " + err)

    reconnectChat()
        integer conn = live.open("wss://chat.example.com/room", "onChatMessage", "onChatClosed", "onChatError")
        integer s = ui.setState("conn", conn.toString())

    sendMessage()
        string msg = ui.inputValue("#msg-input")
        string connStr = ui.getState("conn")
        integer conn = connStr.toInteger()
        integer s = live.send(conn, msg)
        s = ui.setInput("#msg-input", "")
```

### 8.5 Live Data Dashboard

```clean
import frame.ui
import frame.client

start:
    integer conn = live.open("wss://api.example.com/metrics", "onMetrics", "onMetricsLost", "onMetricsError")

functions:
    onMetrics()
        string data = live.message()
        string cpu = json.get(data, "cpu")
        string mem = json.get(data, "memory")
        string reqs = json.get(data, "requestsPerSec")
        integer s = ui.updateElement("#cpu", cpu + "%")
        s = ui.updateElement("#mem", mem + "%")
        s = ui.updateElement("#reqs", reqs + "/s")
        s = ui.updateAttr("#cpu-bar", "style", "width:" + cpu + "%")
        s = ui.updateAttr("#mem-bar", "style", "width:" + mem + "%")

    onMetricsLost()
        integer s = ui.addClass("#dashboard", "offline")
        s = ui.setTimeout("reconnectMetrics", 3000)

    onMetricsError()
        integer s = ui.addClass("#dashboard", "error")

    reconnectMetrics()
        integer conn = live.open("wss://api.example.com/metrics", "onMetrics", "onMetricsLost", "onMetricsError")
        integer s = ui.removeClass("#dashboard", "offline")
```

### 8.6 Progress Streaming with Feed

```clean
import frame.ui
import frame.client

start:
    integer conn = feed.open("/api/build/stream", "onBuildLog", "onBuildFeedError")
    integer s = feed.on(conn, "progress", "onBuildProgress")
    s = feed.on(conn, "complete", "onBuildComplete")
    s = feed.on(conn, "error", "onBuildFailed")
    s = ui.setState("feed-conn", conn.toString())

functions:
    onBuildLog()
        string data = feed.data()
        integer s = ui.updateElement("#log", data)

    onBuildFeedError()
        integer s = ui.addClass("#feed-icon", "reconnecting")

    onBuildProgress()
        string data = feed.data()
        string pct = json.get(data, "percent")
        integer s = ui.updateAttr("#progress", "style", "width:" + pct + "%")
        s = ui.updateElement("#pct-text", pct + "%")

    onBuildComplete()
        string connStr = ui.getState("feed-conn")
        integer conn = connStr.toInteger()
        integer s = feed.close(conn)
        s = ui.addClass("#progress", "complete")
        s = ui.updateElement("#pct-text", "Done!")

    onBuildFailed()
        string data = feed.data()
        string msg = json.get(data, "message")
        integer s = ui.addClass("#progress", "failed")
        s = ui.updateElement("#pct-text", "Failed: " + msg)
```

### 8.7 Live Notifications

```clean
import frame.ui
import frame.client

start:
    integer conn = feed.open("/api/notifications", "onNotification", "onNotificationError")
    integer s = feed.on(conn, "info", "onInfoNotification")
    s = feed.on(conn, "warning", "onWarningNotification")
    s = feed.on(conn, "error", "onErrorNotification")

functions:
    onNotification()
        string msg = feed.data()
        integer s = ui.updateElement("#toast", msg)
        s = ui.addClass("#toast", "visible")
        s = ui.setTimeout("hideToast", 3000)

    onNotificationError()
        integer s = ui.addClass("#feed-status", "reconnecting")

    onInfoNotification()
        string data = feed.data()
        string msg = json.get(data, "message")
        integer s = ui.updateElement("#toast", msg)
        s = ui.addClass("#toast", "visible")
        s = ui.addClass("#toast", "toast-info")
        s = ui.setTimeout("hideToast", 3000)

    onWarningNotification()
        string data = feed.data()
        string msg = json.get(data, "message")
        integer s = ui.updateElement("#toast", msg)
        s = ui.addClass("#toast", "visible")
        s = ui.addClass("#toast", "toast-warning")
        s = ui.setTimeout("hideToast", 5000)

    onErrorNotification()
        string data = feed.data()
        string msg = json.get(data, "message")
        integer s = ui.updateElement("#toast", msg)
        s = ui.addClass("#toast", "visible")
        s = ui.addClass("#toast", "toast-error")
        s = ui.setTimeout("hideToast", 8000)

    hideToast()
        integer s = ui.removeClass("#toast", "visible")
        s = ui.removeClass("#toast", "toast-info")
        s = ui.removeClass("#toast", "toast-warning")
        s = ui.removeClass("#toast", "toast-error")
```

### 8.8 Polling Pattern

```clean
import frame.ui
import frame.client

start:
    integer s = api.get("/api/stats", "onStats")

functions:
    onStats()
        if api.ok() == 1
            string users = api.json("activeUsers")
            integer s = ui.updateElement("#active", users)
        integer s = ui.setTimeout("pollStats", 30000)

    pollStats()
        integer s = api.get("/api/stats", "onStats")
```

### 8.9 Mixed: API + Live + Feed

```clean
import frame.ui
import frame.client

start:
    integer s = api.auth("bearer", "my-token")
    s = api.get("/api/dashboard", "onDashboard")
    integer ws = live.open("wss://api.example.com/alerts", "onAlert", "onAlertLost", "onAlertError")
    integer sse = feed.open("/api/activity", "onActivity", "onActivityError")

functions:
    onDashboard()
        if api.ok() == 1
            string total = api.json("stats.totalUsers")
            integer s = ui.updateElement("#total-users", total)

    onAlert()
        string data = live.message()
        string level = json.get(data, "level")
        string msg = json.get(data, "message")
        integer s = ui.updateElement("#alert", msg)
        s = ui.addClass("#alert", "alert-" + level)

    onAlertLost()
        integer s = ui.addClass("#ws-indicator", "offline")

    onAlertError()
        integer s = ui.updateElement("#alert", "Connection error")

    onActivity()
        string data = feed.data()
        string action = json.get(data, "action")
        string user = json.get(data, "user")
        string current = ui.getText("#activity-log")
        integer s = ui.updateElement("#activity-log", "<div>" + user + " " + action + "</div>" + current)

    onActivityError()
        integer s = ui.addClass("#sse-indicator", "reconnecting")
```

---

## 9. Security

### 9.1 CORS
Browser CORS policy applies. Server must send `Access-Control-Allow-*` headers for cross-origin requests.

### 9.2 Credentials
- `api.auth()` stores in JS memory. Cleared on page unload or `api.clearAuth()`.
- Auth applied automatically to all `api.*` calls.
- `api.header()` sets per-request only, no auto-auth.
- Cookies follow default `credentials: 'same-origin'`.

### 9.3 WebSocket/SSE
Server-side origin validation is the developer's responsibility.

---

## 10. Context Isolation

Each protocol has its own context. No ambiguity:

| Context Function | Available When |
|-----------------|----------------|
| `ui.eventValue()`, `ui.eventAttr()` | Inside DOM event handler |
| `api.status()`, `api.body()`, `api.json()` | Inside API response handler |
| `live.message()`, `live.connId()` | Inside live message handler |
| `live.closeCode()`, `live.closeReason()` | Inside live close handler |
| `live.error()` | Inside live error handler |
| `feed.data()`, `feed.eventType()` | Inside feed event handler |

---

## 11. Function Summary

### frame.ui additions: 5 functions

| Function | Description |
|----------|-------------|
| `ui.inputValue(selector)` | Read input value |
| `ui.formJson(selector)` | Collect form as JSON |
| `ui.formData(selector)` | Collect form as URL-encoded |
| `ui.checked(selector)` | Checkbox state |
| `ui.setInput(selector, value)` | Set input value |

### frame.client: 30 functions

| Namespace | Count | Functions |
|-----------|-------|-----------|
| `api.*` | 15 | get, post, put, patch, delete, header, timeout, auth, clearAuth, submit, status, body, ok, json, responseHeader |
| `live.*` | 9 | open, send, close, state, message, connId, closeCode, closeReason, error |
| `feed.*` | 7 | open, on, close, data, eventType, lastId, connId |

**Total new functions: 35**

### Handler Type

All functions accepting callbacks use the `handler` param type. The compiler resolves function names to indices automatically. The developer writes named functions, never numbers. See cross-component prompt: `compiler-handler-type-for-plugins.md`.

---

## 12. Implementation Priority

| Phase | What | Count | Rationale |
|-------|------|-------|-----------|
| **Phase 1** | Form helpers (frame.ui) + API (frame.client) | 20 | Unblocks 80% of use cases |
| **Phase 2** | Live / WebSocket | 9 | Real-time communication |
| **Phase 3** | Feed / SSE | 7 | Server push |

---

## 13. Files

| Action | File |
|--------|------|
| **Create** | `plugins/frame.client/plugin.toml` |
| **Create** | `plugins/frame.client/runtime/loader.js` |
| **Modify** | `plugins/frame.ui/plugin.toml` -- add 5 form helpers |
| **Modify** | `plugins/frame.ui/runtime/loader.js` -- add form helper implementations |
| **Rename** | `plugins/frame.server/` → `plugins/frame.server/` |

### Cross-Component

| Component | Change |
|-----------|--------|
| Build system | Merge plugin runtimes into unified loader.js |
| Compiler | Verify multi-namespace plugin support (should work with `expand_strings`) |

---

**End of Document 14**
