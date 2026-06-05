# Frame Client Communication Specification (14)

**Project:** Frame -- Full-Stack Framework for Clean Language
**Version:** 2.2
**Status:** Specification
**Location:** `/documents/specification/14_frame_ui_client_communication.md`

---

## 1. Overview

This specification defines **frame.client** -- the plugin for all client-side communication between the browser and the server. It provides three namespaces and one declarative block:

| Namespace / Block | Protocol | What it does |
|-------------------|----------|-------------|
| `api.*` | HTTP | Call APIs, submit forms, read responses |
| `live.*` | WebSocket | Bidirectional real-time connections |
| `feed.*` | SSE | Server-push event streams |
| `load:` | HTTP | Declarative page-level data fetching — generates module state + start: fetch |

It also specifies form-reading helpers added to **frame.ui**.

---

## 2. Architecture

### 2.1 Plugin Map

```
frame.ui      → What you SEE       (DOM, events, components, state, timers)
frame.client  → What you SEND      (api, live, feed, load: — all client-server communication)
frame.server  → What you SERVE     (endpoints, request context, response helpers)
frame.data    → What you STORE     (models, queries, migrations)
frame.auth    → What you SECURE    (login, sessions, roles)
frame.canvas  → What you DRAW      (graphics, animation, sprites)
```

### 2.2 Why frame.client

| Principle | Decision |
|-----------|----------|
| Plugin name = what it does | `client` = "I'm the client talking to the server" |
| No namespace collision | Server uses `http.*` (sync outgoing). Client uses `api.*` (async browser). No overlap. |
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

// Client-side (frame.client): async, use later to suspend until response arrives
later result = api.get("/api/users")
if result.ok
    string name = result.json("name")
```

Different plugins. Different namespaces. Different calling conventions. Clear.

### 2.5 Plugin Naming

Developer-facing syntax for `frame.server` is unchanged: `endpoints:`, `req.*`, `json()`, `redirect()`. See [03_frame_server.md](03_frame_server.md) for the full server API.

---

## 3. Async Model: `later` + Result Object

All `api.*` request functions return a `Future<ApiResponse>`. Use the `later` keyword to suspend the current handler until the response arrives, then read the result directly:

```clean
events:
    onLoadUsers:
        later result = api.get("/api/users")
        if result.ok
            state.users = result.json("data")
        else
            state.error = "Failed to load users"
```

`later` is a native Clean Language keyword that suspends the current coroutine until the expression resolves. The component re-renders automatically when state changes after the `later` line resumes.

### 3.1 ApiResponse Object

Every `api.*` request returns an `ApiResponse` with these properties and methods:

| Access | Type | Description |
|--------|------|-------------|
| `result.ok` | `boolean` | True if HTTP status is 200-299 |
| `result.status` | `integer` | HTTP status code (0 = network error) |
| `result.body` | `string` | Response body as a raw string |
| `result.json("path")` | `any` | Extract a value from the JSON body via dot-notation path |
| `result.header("name")` | `string` | Get a response header value |

### 3.2 Why `later` and Not True Synchronous

WASM in the browser cannot block the main thread. `later` uses coroutine suspension -- the component's event handler is paused at the `later` line and the browser remains responsive. When the HTTP response arrives, the handler resumes from that exact point with the result populated.

SharedArrayBuffer + Atomics could block WASM synchronously but was rejected:
- Requires COOP/COEP headers (breaks iframes, CDNs, embeds)
- Needs a Web Worker (bundle size, complexity)
- Incompatible with some mobile/WebView environments

`later` gives the same readable linear code without any of those constraints.

### 3.3 `live.*` and `feed.*` Still Use Named Handlers

`later` is only for `api.*` (one request, one response). WebSocket and SSE connections receive an indefinite stream of events, so they continue to use named handler functions:

```clean
// api.* — one shot, use later
later result = api.get("/api/data")

// live.* — persistent stream, use named handlers
integer conn = live.open("wss://example.com/ws", "onMessage", "onClose", "onError")

// feed.* — persistent stream, use named handlers
integer conn = feed.open("/api/events", "onEvent", "onError")
```

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

All request functions return `Future<ApiResponse>`. Use `later result = api.get(url)` to suspend until the response arrives.

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `api.get(url)` | `_api_get` | `string` | `Future<ApiResponse>` | GET request |
| `api.post(url, body)` | `_api_post` | `string, string` | `Future<ApiResponse>` | POST with JSON body |
| `api.put(url, body)` | `_api_put` | `string, string` | `Future<ApiResponse>` | PUT with JSON body |
| `api.patch(url, body)` | `_api_patch` | `string, string` | `Future<ApiResponse>` | PATCH with JSON body |
| `api.delete(url)` | `_api_delete` | `string` | `Future<ApiResponse>` | DELETE request |
| `api.submit(formSelector, url, method)` | `_api_submit` | `string, string, string` | `Future<ApiResponse>` | Collect form JSON + send |

### 5.2 Request Configuration

These are called before the request to set headers, auth, or timeout. They take effect for the next request only (except `api.auth` which persists).

| Clean Syntax | Bridge Name | Params | Returns | Description |
|-------------|-------------|--------|---------|-------------|
| `api.header(name, value)` | `_api_header` | `string, string` | `integer` | Set header for next request |
| `api.timeout(ms)` | `_api_timeout` | `integer` | `integer` | Set timeout for next request |
| `api.auth(scheme, credential)` | `_api_auth` | `string, string` | `integer` | Set auth for all requests |
| `api.clearAuth()` | `_api_clearAuth` | none | `integer` | Clear stored auth |

### 5.3 ApiResponse Properties and Methods

Read these on the result after `later` resolves:

| Access | Type | Description |
|--------|------|-------------|
| `result.ok` | `boolean` | True if HTTP status 200-299 |
| `result.status` | `integer` | HTTP status code (0 = network error) |
| `result.body` | `string` | Response body as string |
| `result.json("path")` | `any` | Extract value from JSON body via dot-notation |
| `result.header("name")` | `string` | Get response header value |

### 5.4 Error Handling

| Scenario | `result.ok` | `result.status` | `result.body` |
|----------|-------------|-----------------|---------------|
| Success (200) | `true` | `200` | Response body |
| Not found (404) | `false` | `404` | Error body from server |
| Server error (500) | `false` | `500` | Error body from server |
| Network failure | `false` | `0` | `{"error": "Failed to fetch"}` |
| Timeout | `false` | `0` | `{"error": "The operation was aborted."}` |
| CORS blocked | `false` | `0` | `{"error": "...CORS..."}` |

### 5.5 Auth Persistence

```clean
// Set once -- applies to all subsequent api.* calls
api.auth("bearer", token)

// These all include Authorization header automatically:
later r1 = api.get("/protected/data")
later r2 = api.post("/protected/create", body)
later r3 = api.submit("#form", "/protected/save", "POST")

// Clear when done
api.clearAuth()
```

Manual override for one request: call `api.header("Authorization", "Bearer " + token)` before the request.

### 5.6 `api.submit()` -- Convenience

Collects form fields as JSON, sets Content-Type, and sends. Returns the same `Future<ApiResponse>`:

```clean
// Long way:
events:
    onCreateUser:
        string body = ui.formJson("#create-form")
        integer s = api.header("Content-Type", "application/json")
        later result = api.post("/api/users", body)
        if result.ok
            state.created = true

// Short way:
events:
    onCreateUser:
        later result = api.submit("#create-form", "/api/users", "POST")
        if result.ok
            state.created = true
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
component: tag="user-profile"
    inputs:
        integer userId

    state:
        string name = ""
        string error = ""

    events:
        onLoad:
            later result = api.get("/api/users/" + inputs.userId.toString())
            if result.ok
                state.name = result.json("name")
            else
                state.error = "Failed to load user"

    html:
        <div cl-if="state.error != ''" class="error">{state.error}</div>
        <div cl-if="state.name != ''">{state.name}</div>
```

### 8.2 Authenticated CRUD

```clean
component: tag="user-list"
    state:
        list<any> users = []
        string error = ""

    events:
        onLoad:
            api.auth("bearer", state.token)
            later result = api.get("/api/users")
            if result.ok
                state.users = result.json("data")

        onDeleteUser:
            string id = ui.eventAttr("data-id")
            later result = api.delete("/api/users/" + id)
            if result.ok
                later reload = api.get("/api/users")
                if reload.ok
                    state.users = reload.json("data")

        onCreateUser:
            later result = api.submit("#create-form", "/api/users", "POST")
            if result.status == 201
                later reload = api.get("/api/users")
                if reload.ok
                    state.users = reload.json("data")
            else
                state.error = result.json("message")

    html:
        <div cl-iterate="user in state.users">
            <span>{user.name}</span>
            <button data-id="{user.id}" cl-on:click="onDeleteUser">Delete</button>
        </div>
        <form id="create-form">
            <input name="name" />
            <input name="email" />
            <button type="button" cl-on:click="onCreateUser">Create</button>
        </form>
```

### 8.3 Custom Headers

```clean
component: tag="legacy-client"
    state:
        string result = ""

    events:
        onSend:
            integer s = api.header("X-Api-Key", "secret-123")
            s = api.header("Content-Type", "text/xml")
            s = api.timeout(5000)
            later r = api.post("/api/legacy", "<xml>data</xml>")
            state.result = r.body

    html:
        <button cl-on:click="onSend">Send</button>
        <pre>{state.result}</pre>
```

### 8.4 Live Chat

```clean
component: tag="chat-room"
    state:
        string messages = ""
        integer connId = 0

    events:
        onLoad:
            integer conn = live.open("wss://chat.example.com/room", "onChatMessage", "onChatClosed", "onChatError")
            state.connId = conn

        onSend:
            string msg = ui.inputValue("#msg-input")
            integer s = live.send(state.connId, msg)
            s = ui.setInput("#msg-input", "")

        onChatMessage:
            string data = live.message()
            string user = _json_get(data, "user")
            string msg = _json_get(data, "message")
            state.messages = state.messages + "<div><b>" + user + ":</b> " + msg + "</div>"

        onChatClosed:
            integer code = live.closeCode()
            state.messages = state.messages + "<div class='system'>Disconnected</div>"

        onChatError:
            string err = live.error()
            state.messages = state.messages + "<div class='error'>Error: " + err + "</div>"

    html:
        <div id="messages">{!state.messages}</div>
        <input id="msg-input" />
        <button cl-on:click="onSend">Send</button>
```

### 8.5 Live Data Dashboard

```clean
component: tag="metrics-dashboard"
    state:
        string cpu = "0"
        string mem = "0"
        string reqs = "0"
        boolean offline = false

    events:
        onLoad:
            integer conn = live.open("wss://api.example.com/metrics", "onMetrics", "onMetricsLost", "onMetricsError")

        onMetrics:
            string data = live.message()
            state.cpu = _json_get(data, "cpu")
            state.mem = _json_get(data, "memory")
            state.reqs = _json_get(data, "requestsPerSec")
            state.offline = false

        onMetricsLost:
            state.offline = true

        onMetricsError:
            state.offline = true

    html:
        <div cl-if="state.offline" class="offline-banner">Reconnecting...</div>
        <div>CPU: {state.cpu}%</div>
        <div>Memory: {state.mem}%</div>
        <div>Requests/s: {state.reqs}</div>
```

### 8.6 Progress Streaming with Feed

```clean
component: tag="build-progress"
    state:
        string log = ""
        integer percent = 0
        boolean complete = false
        boolean failed = false

    events:
        onLoad:
            integer conn = feed.open("/api/build/stream", "onBuildLog", "onBuildFeedError")
            integer s = feed.on(conn, "progress", "onBuildProgress")
            s = feed.on(conn, "complete", "onBuildComplete")
            s = feed.on(conn, "error", "onBuildFailed")

        onBuildLog:
            state.log = state.log + feed.data() + "\n"

        onBuildProgress:
            string data = feed.data()
            state.percent = _json_get(data, "percent").toInteger()

        onBuildComplete:
            state.complete = true
            state.percent = 100

        onBuildFailed:
            state.failed = true

        onBuildFeedError:
            state.log = state.log + "[connection lost, reconnecting...]\n"

    html:
        <div class="progress-bar" style="width:{state.percent}%"></div>
        <pre>{state.log}</pre>
```

### 8.7 Polling Pattern

```clean
component: tag="stats-counter"
    state:
        string activeUsers = "0"

    events:
        onLoad:
            later result = api.get("/api/stats")
            if result.ok
                state.activeUsers = result.json("activeUsers")
            integer s = ui.setTimeout("onLoad", 30000)

    html:
        <span>{state.activeUsers} active users</span>
```

### 8.8 Mixed: API + Live + Feed

```clean
component: tag="dashboard"
    state:
        string totalUsers = "0"
        string alertMsg = ""
        string lastActivity = ""

    events:
        onLoad:
            api.auth("bearer", state.token)
            later result = api.get("/api/dashboard")
            if result.ok
                state.totalUsers = result.json("stats.totalUsers")
            integer ws = live.open("wss://api.example.com/alerts", "onAlert", "onAlertLost", "onAlertError")
            integer sse = feed.open("/api/activity", "onActivity", "onActivityError")

        onAlert:
            string data = live.message()
            state.alertMsg = _json_get(data, "message")

        onAlertLost:
            state.alertMsg = "Alert connection lost"

        onAlertError:
            state.alertMsg = "Alert connection error"

        onActivity:
            string data = feed.data()
            string action = _json_get(data, "action")
            string user = _json_get(data, "user")
            state.lastActivity = user + " " + action

        onActivityError:
            state.lastActivity = "Activity feed reconnecting..."

    html:
        <div>Total users: {state.totalUsers}</div>
        <div cl-if="state.alertMsg != ''">{state.alertMsg}</div>
        <div>{state.lastActivity}</div>
```

---

## 9. Declarative Client Data Fetching (`load:` Block)

The `load:` block is a top-level block owned by **frame.client**. It lives in a companion `.cln` file alongside a page (not nested inside a component) and fetches data from APIs when the page loads in the browser.

It generates module-level state variables and a `start:` block — so the fetch runs automatically when the client WASM module loads, making data available to every component on the page.

This mirrors the server-side pattern: just as `load()` in a companion file provides data for SSR rendering, `load:` in a client companion file provides data for client-side rendering.

### 9.1 Syntax

```
load:
    <name> from "<url>" [at "<json-path>"] [on error: "<handler>"]
    ...
```

| Part | Required | Description |
|------|----------|-------------|
| `<name>` | yes | Variable name for the parsed response data |
| `from "<url>"` | yes | URL to fetch. May use `{query.X}` to interpolate URL query parameters. |
| `at "<json-path>"` | no | Dot-notation path to extract from the JSON body. Omit to use the full body. |
| `on error: "<handler>"` | no | Name of an exported function to call on failure. |

### 9.2 Where It Lives

`load:` belongs in a client-side companion `.cln` file:

```
app/web/pages/
├── users.html          ← HTML template
├── users.cln           ← server-side: load(), guard()
└── users.client.cln    ← client-side: load: block (frame.client)
```

Or in a standalone client file in `app/web/client/`:

```
app/web/client/
└── users-data.cln      ← load: block only
```

### 9.3 What It Generates

```clean
load:
    users from "/api/users" at "data"
    stats from "/api/stats"
```

Expands to:

```clean
boolean loading = true
string loadError = ""
any users = null
any stats = null

start:
    later __res_users = api.get("/api/users")
    if __res_users.ok
        users = __res_users.json("data")
    else
        loadError = "Failed to load users"
    later __res_stats = api.get("/api/stats")
    if __res_stats.ok
        stats = __res_stats.json()
    else
        loadError = "Failed to load stats"
    loading = false
```

### 9.4 Generated Variables

| Variable | Type | Value |
|----------|------|-------|
| `<name>` | `any` | Parsed JSON response (or extracted path). `null` until fetch completes. |
| `loading` | `boolean` | `true` while any fetch is pending. `false` when all complete. |
| `loadError` | `string` | Error message from the last failed fetch. `""` on success. |

### 9.5 URL Query Parameter Interpolation

Use `{query.X}` to inject URL query parameters into the fetch URL:

```clean
load:
    posts from "/api/posts?category={query.category}" at "items"
```

The runtime reads the value of `?category=` from the current page URL and substitutes it.

### 9.6 Multiple Declarations

All fetches run sequentially. `loading` stays `true` until all complete.

```clean
load:
    user from "/api/me" at "data"
    posts from "/api/me/posts" at "items"
    stats from "/api/me/stats"
```

`loadError` holds the message from the last failure (if any).

### 9.7 Custom Error Handling

```clean
load:
    config from "/api/config" on error: "handleConfigError"
```

`handleConfigError()` must be an exported function in the same module.

### 9.8 Comparison with Component-Level Fetching

For data needed by a single component only, use `api.*` directly inside the component's `onLoad` event handler. The `load:` block is for **page-level** data shared across multiple components.

| | `load:` block | `api.*` in component |
|---|---|---|
| Scope | Whole page | One component |
| Trigger | Page load (automatic) | Component event handler |
| Data sharing | All components on page | Private to component |
| Boilerplate | Zero | Manual state + fetch |

---

## 10. Declarative Form Wiring (`form:` Block)

The `form:` block is a top-level block owned by **frame.client**. It lives in a companion `.cln` file alongside a page and generates all the boilerplate needed for a form submission: field state variables, a `submitting` flag, error/success strings, and a `submit()` function.

### 10.1 Syntax

```
form: <method> "<url>"
    <field> [required] [default="<value>"]
    ...
    [on success: [clear] [emit "<handler>"]]
    [on error: show]
```

| Part | Required | Description |
|------|----------|-------------|
| `<method>` | yes | HTTP method: `post`, `put`, `patch`, `get`, or `delete` (case-insensitive). |
| `"<url>"` | yes | Endpoint URL string literal. |
| `<field>` | at least one | Name of a string state variable to generate. |
| `required` | no | Validates the field is non-empty before submitting. |
| `default="<value>"` | no | Initial value for the field (default: `""`). |
| `on success: clear` | no | Reset all fields to their defaults after a successful submit. |
| `on success: emit "<handler>"` | no | Call the named exported function after success. |
| `on error: show` | no | Set `formError` from the response JSON `"error"` key (default behaviour). |

### 10.2 Where It Lives

`form:` belongs in a client-side companion `.cln` file:

```
app/web/pages/
├── new-task.html            ← HTML template
├── new-task.cln             ← server-side: guard(), load()
└── new-task.client.cln     ← client-side: form: block (frame.client)
```

### 10.3 What It Generates

```clean
form: post "/api/tasks"
    title required
    description default="Enter description"
    on success: clear emit "reloadTasks"
```

Expands to:

```clean
string title = ""
string description = "Enter description"
boolean submitting = false
string formError = ""
string formSuccess = ""

void submit()
    if submitting
        return
    if title.trim() == ""
        formError = "title is required"
        return
    submitting = true
    formError = ""
    formSuccess = ""
    string __body = "{\"title\":\"" + title + "\",\"description\":\"" + description + "\"}"
    later __res = api.post("/api/tasks", __body)
    if __res.ok
        title = ""
        description = "Enter description"
        formSuccess = "Saved"
        reloadTasks()
    else
        formError = __res.json("error")
        if formError == ""
            formError = "Request failed"
    submitting = false
```

### 10.4 Generated Variables

| Variable | Type | Initial value | Purpose |
|----------|------|---------------|---------|
| `<field>` | `string` | `""` or `default="…"` value | One variable per declared field. |
| `submitting` | `boolean` | `false` | `true` while the request is in flight. Use to disable the submit button. |
| `formError` | `string` | `""` | Set on validation failure or API error. |
| `formSuccess` | `string` | `""` | Set to `"Saved"` on success. |

### 10.5 Generated Function

`void submit()` is generated at module level and can be called from any component on the page via `emit`. It is idempotent: if `submitting` is already `true` it returns immediately.

### 10.6 Methods Without a Body

For `get` and `delete`, no `__body` variable is generated and the API call omits the body argument:

```clean
form: delete "/api/tasks/{id}"
    id required
```

Generates `api.delete("/api/tasks/{id}")` — note the `id` field is used for the URL, not sent as JSON.

### 10.7 Comparison with `api.submit()`

| | `form:` block | `api.submit()` |
|---|---|---|
| State management | Automatic | Manual |
| Validation | Declarative (`required`) | Manual |
| `submitting` guard | Built-in | Manual |
| Error state | Built-in `formError` | Manual |
| JSON body | Built-in (field variables) | Collects from DOM inputs |
| Use when | Page-level form, fields in state | One-shot submit of an HTML form |

---

## 11. Security

### 11.1 CORS
Browser CORS policy applies. Server must send `Access-Control-Allow-*` headers for cross-origin requests.

### 11.2 Credentials
- `api.auth()` stores in JS memory. Cleared on page unload or `api.clearAuth()`.
- Auth applied automatically to all `api.*` calls.
- `api.header()` sets per-request only, no auto-auth.
- Cookies follow default `credentials: 'same-origin'`.

### 11.3 WebSocket/SSE
Server-side origin validation is the developer's responsibility.

---

## 12. Context Isolation

| Context | Available When |
|---------|----------------|
| `ui.eventValue()`, `ui.eventAttr()` | Inside a DOM event handler |
| `result.ok`, `result.status`, `result.body`, `result.json()` | On an `ApiResponse` after `later` resolves |
| `live.message()`, `live.connId()` | Inside a live message handler |
| `live.closeCode()`, `live.closeReason()` | Inside a live close handler |
| `live.error()` | Inside a live error handler |
| `feed.data()`, `feed.eventType()` | Inside a feed event handler |

---

## 13. Function Summary

### frame.ui additions: 5 functions

| Function | Description |
|----------|-------------|
| `ui.inputValue(selector)` | Read input value |
| `ui.formJson(selector)` | Collect form as JSON |
| `ui.formData(selector)` | Collect form as URL-encoded |
| `ui.checked(selector)` | Checkbox state |
| `ui.setInput(selector, value)` | Set input value |

### frame.client: 27 functions + 2 blocks

| Namespace / Block | Count | Functions / Keywords |
|-------------------|-------|----------------------|
| `api.*` requests | 5 | get, post, put, patch, delete |
| `api.*` config | 4 | header, timeout, auth, clearAuth |
| `api.*` convenience | 1 | submit |
| `live.*` | 9 | open, send, close, state, message, connId, closeCode, closeReason, error |
| `feed.*` | 7 | open, on, close, data, eventType, lastId, connId |
| `load:` block | — | Declarative page-level data fetching; generates `loading`, `loadError`, named `any` variables. See §9. |
| `form:` block | — | Declarative form wiring; generates field variables, `submitting`, `formError`, `formSuccess`, `submit()`. See §10. |

**ApiResponse** — returned by all `api.*` request functions, read via `later result = api.*`:

| Property/Method | Description |
|-----------------|-------------|
| `result.ok` | True if 200-299 |
| `result.status` | HTTP status code |
| `result.body` | Raw response string |
| `result.json("path")` | JSON value extraction |
| `result.header("name")` | Response header value |

---

## 14. Implementation Priority

| Phase | What | Count | Rationale |
|-------|------|-------|-----------|
| **Phase 1** | Form helpers (frame.ui) + API (frame.client) | 15 | Unblocks 80% of use cases |
| **Phase 2** | Live / WebSocket | 9 | Real-time communication |
| **Phase 3** | Feed / SSE | 7 | Server push |
| **Phase 4** | `load:` block | — | Declarative sugar over Phase 1 API; requires Phase 1 complete |
| **Phase 5** | `form:` block | — | Declarative sugar over Phase 1 API + state management |

---

## 15. Files

| Action | File |
|--------|------|
| **Create** | `plugins/frame.client/plugin.toml` |
| **Create** | `plugins/frame.client/runtime/loader.js` |
| **Modify** | `plugins/frame.ui/plugin.toml` -- add 5 form helpers |
| **Modify** | `plugins/frame.ui/runtime/loader.js` -- add form helper implementations |

### Cross-Component

| Component | Change |
|-----------|--------|
| Build system | Merge plugin runtimes into unified loader.js |
| Compiler | `later` keyword support for `Future<T>` return values from bridge functions |
| Compiler | `load:` block expansion — parse block, generate `api.get()` + state management + reactive re-fetch on inputs change |

---

**End of Document 14**
