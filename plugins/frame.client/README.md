# frame.client

Client-side communication plugin for Clean Language. Provides HTTP API calls, WebSocket live connections, and Server-Sent Events feeds.

## Namespaces

| Namespace | Protocol | Purpose |
|-----------|----------|---------|
| `api.*` | HTTP | Call APIs, submit forms, read responses |
| `live.*` | WebSocket | Bidirectional real-time connections |
| `feed.*` | SSE | Server-push event streams |

## Usage

```clean
import frame.ui
import frame.client

start:
    integer s = ui.onEvent("#load-btn", "click", loadUsers)

functions:
    loadUsers()
        integer s = api.get("/api/users", onUsersLoaded)

    onUsersLoaded()
        if api.ok() == 1
            string name = api.json("name")
            integer s = ui.updateElement("#user-name", name)
```

## API Functions (HTTP)

### Request Functions

| Function | Description |
|----------|-------------|
| `api.get(url, handler)` | GET request |
| `api.post(url, body, handler)` | POST request |
| `api.put(url, body, handler)` | PUT request |
| `api.patch(url, body, handler)` | PATCH request |
| `api.delete(url, handler)` | DELETE request |
| `api.header(name, value)` | Set header for next request |
| `api.timeout(ms)` | Set timeout for next request |
| `api.auth(scheme, credential)` | Set auth for all requests |
| `api.clearAuth()` | Clear stored auth |
| `api.submit(formSelector, url, method, handler)` | Collect form + send |

### Response Context (inside handler)

| Function | Description |
|----------|-------------|
| `api.status()` | HTTP status code (0 = network error) |
| `api.body()` | Response body string |
| `api.ok()` | 1 if 200-299, 0 otherwise |
| `api.json(path)` | Extract JSON value by dot-path |
| `api.responseHeader(name)` | Response header value |

## Live Functions (WebSocket)

| Function | Description |
|----------|-------------|
| `live.open(url, onMsg, onClose, onError)` | Open connection, returns ID |
| `live.send(connId, message)` | Send message |
| `live.close(connId)` | Close connection |
| `live.state(connId)` | "connecting", "open", "closing", "closed" |
| `live.message()` | Message data (in handler) |
| `live.connId()` | Connection ID (in handler) |
| `live.closeCode()` | Close code (in handler) |
| `live.closeReason()` | Close reason (in handler) |
| `live.error()` | Error message (in handler) |

## Feed Functions (SSE)

| Function | Description |
|----------|-------------|
| `feed.open(url, onMsg, onError)` | Open feed, returns ID |
| `feed.on(connId, eventName, handler)` | Listen for named event |
| `feed.close(connId)` | Close feed |
| `feed.data()` | Event data (in handler) |
| `feed.eventType()` | Event type name (in handler) |
| `feed.lastId()` | Last event ID (in handler) |
| `feed.connId()` | Connection ID (in handler) |

## Form Helpers

Form reading helpers are in **frame.ui** (not frame.client), since reading DOM inputs is a UI operation:

| Function | Description |
|----------|-------------|
| `ui.inputValue(selector)` | Read input value |
| `ui.formJson(selector)` | Collect form as JSON string |
| `ui.formData(selector)` | Collect form as URL-encoded |
| `ui.checked(selector)` | Checkbox state (0/1) |
| `ui.setInput(selector, value)` | Set input value |

## Runtime Architecture

frame.client is a bridge-only plugin. It has no `src/main.cln` or `plugin.wasm`. Its runtime (`bridge.js`) registers bridge functions into `__cleanRuntime` before WASM instantiation.

Load order in HTML:
```html
<script src="bridge.js"></script>
<script src="loader.js" data-wasm="frontend.wasm"></script>
```

## Specification

Full specification: [14_frame_ui_client_communication.md](../../documents/specification/14_frame_ui_client_communication.md)
