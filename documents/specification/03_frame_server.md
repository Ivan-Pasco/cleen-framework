# Frame Server Specification (03)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.3 (inline route modifiers: `[guard] cache() middleware()`)  
**Location (repo):** `/documents/specification/03_frame_server.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

## 1. Purpose
The Frame Server runs the backend WASM, routes HTTP requests, bridges Clean code to the Host via **Host Bridge**, and renders SSR HTML for Frame UI. This version standardizes API declaration using a single, declarative block: **`endpoints:`**.

**Goals**
- One obvious way to declare HTTP APIs.
- Typed inputs/outputs; effortless OpenAPI generation.
- Block-first, indentation-based, readable by humans and tools.

---

## 2. Responsibilities
- Load and execute `backend.wasm`.
- HTTP routing and request parsing (params, query, headers, body, forms).
- JSON serialization (`json()`), redirects, error helpers.
- Session/auth integration.
- SSR pipeline for Frame UI.
- Logging, timing, and environment access through Host Bridge.

---

## 3. File Layout

> **Canonical reference:** [PROJECT_STRUCTURE.md](../PROJECT_STRUCTURE.md) — complete folder reference.

```
/app/server/             # Owned by frame.server plugin
/app/server/api/*.cln    # API modules using `endpoints:`
/app/server/middleware/  # Custom middleware
/app/logic/              # Business logic services
/app/web/pages/*.html    # SSR page templates (HTML)
/app/web/pages/*.cln     # Companion data loaders (paired by filename)
/app/web/routes.cln      # Routing file: guards, redirects, rewrites, error pages
/app/web/components/*.cln # UI components
/app/web/layouts/*.html  # Page layout wrappers
/app/data/*.cln          # Data models / ORM
/app/auth/*.cln          # Auth configuration (frame.auth)
/public/*                # Static assets
```

**Plugin Folder Ownership:** Files placed in `app/server/` or `app/server/api/` are processed by the `frame.server` plugin. The plugin must be declared in `main.cln` via the `target:` block. Once declared, individual source files in plugin-owned folders do not need their own `import` statement — the folder location determines which plugin processes them.

---

## 4. Declaring HTTP APIs with `endpoints:`
Each API module exposes a single `endpoints:` block. Endpoints are declared by **METHOD + PATH** and the block body handles the request.

```clean
// /app/server/api/users.cln
endpoints:
    GET "/api/users" :
        list<User> users = User.find:
            where:
                active == true
            order:
                createdAt desc
            limit: 50
        return json(users)

    POST "/api/users" :
        CreateUser body = req.json(CreateUser)
        User u = User.insert:
            name  = body.name
            email = body.email
            active = true
        return json(u), status(201)

    GET "/api/users/:id" :
        integer id = req.params.id
        User? u = User.first:
            where:
                id == id
        if u == null
            return notFound()
        return json(u)
```

**Notes**
- Paths are always quoted: `METHOD "/path" :` — this is the only accepted format.
- The handler must `return` a Response (e.g., `json(...)`, `html(...)`, `redirect(...)`).

---

## 5. Inline Route Modifiers
Route metadata lives on the route header line, not in sub-blocks. All modifiers are optional and must appear in this fixed order: **`[guard] cache() middleware()`**.

```clean
endpoints:
    // no modifiers — open route
    GET "/api/public" :
        return json(data)

    // guard only — user must hold at least one listed role
    GET "/api/users" [admin, editor] :
        list<User> users = User.find()
        return json(users)

    // cache only — sets Cache-Control: max-age=300
    GET "/api/feed" cache(300) :
        return json(feed)

    // guard + cache
    GET "/api/reports" [admin] cache("1h") :
        return json(buildReport())

    // guard + cache + middleware
    GET "/api/secure" [admin] cache(60) middleware(RateLimit, Audit) :
        return json(secureData)
```

### `[guard]` — Role-based access
A bracket-delimited list of role identifiers. The server checks the current user holds **at least one** of the listed roles before running the handler. Fails with `401 Unauthorized`.

```clean
GET "/api/admin"         [admin] :          // single role
GET "/api/posts"         [admin, editor] :  // any of these roles
GET "/api/dashboard"     [admin, editor, viewer] :
```

### `cache()` — HTTP cache hints

| Form | Cache-Control emitted |
|------|----------------------|
| `cache(300)` | `max-age=300` |
| `cache("5m")` | `max-age=300` |
| `cache("2h")` | `max-age=7200` |
| `cache("1d")` | `max-age=86400` |
| `cache("no-store")` | `no-store` |
| `cache("no-cache")` | `no-cache` |
| `cache("public")` | `public` |
| `cache("private")` | `private` |

### `middleware()` — Per-route middleware
A comma-separated list of middleware function names. Each is called in order before the handler. A middleware function returns an empty string to pass through or a response string to short-circuit.

```clean
GET "/api/data" middleware(RateLimit) :
GET "/api/admin" [admin] middleware(RateLimit, Audit) :
```

---

## 6. Request Access Helpers
Inside an endpoint/handle block:
```clean
integer id     = req.params.id
string? q      = req.query.q
Headers h      = req.headers
bytes bin      = req.body
CreateUser dto = req.json(CreateUser)
Form form      = req.form
string ip      = req.ip
```

---

## 7. Response Helpers
```clean
return json(data)
return json(data), status(201)
return html("<p>ok</p>")
return redirect("/login")
return notFound()
return badRequest("email already exists")
return unauthorized()
return header("X-Trace", traceId), json(data)
```

---

## 8. Errors (Standard Envelope)
Host Bridge and server utilities use the normalized envelope:
```json
{ "ok": true,  "data": { ... } }
{ "ok": false, "err": { "code": "ERROR_CODE", "message": "...", "details": {} } }
```
Typical codes: `AUTH_ERROR`, `NOT_FOUND`, `VALIDATION_ERROR`, `NETWORK_FAIL`.

---

## 9. Auth Integration
Auth lives in `app/auth/auth.cln`. Use the `[roles]` guard modifier on the route header for declarative role checks, or call auth helpers imperatively inside the handler.

```clean
endpoints:
    GET "/api/admin" [admin] :
        return json({ ok: true })

    GET "/api/content" [admin, editor] :
        return json(content)
```

For permission checks inside a handler:
```clean
if not auth.can(user, "post.publish")
    return unauthorized()
```

---

## 10. OpenAPI & SDK Generation
The server can emit OpenAPI 3.1 by inspecting `endpoints:` declarations, typed params, and DTOs.

```bash
cleen api:spec   # writes openapi.json
cleen api:sdk    # generates Clean/TS/Swift/Kotlin clients
```

**Conventions**
- Response types are inferred from `return` calls (`return json(users)` → array of User).
- Path params (`:id`) are typed from local variable assignments (e.g., `integer id = req.params.id`).

---

## 11. SSR Pipeline (UI)
- Server renders pages from `/app/web/pages/*.html`. Data is supplied by paired companion `.cln` files.
- Output HTML is streamed or buffered (host adapter decides).
- Hydration islands are scheduled according to `client="on|visible|idle|only"`.

---

## 12. Host Bridge (Server‑Side)
The server uses Host Bridge namespaces to perform I/O:

| Namespace | Functions (core) |
|-----------|-------------------|
| `host:http` | `request`, `respond`, `redirect` |
| `host:db`   | `query`, `tx` |
| `host:env`  | `get` |
| `host:time` | `now`, `sleep` |
| `host:crypto` | `random`, `hash`, `verify`, `sign` |
| `host:log`  | `info`, `warn`, `error` |
| `host:fs`   | `read`, `write` *(host-dependent)* |

The Clean code never calls host APIs directly—only via these bridges.

---

## 13. Performance & Logging
- Add timing around hot paths using `host:time.now`.
- Prefer SSR for first paint and stream when host supports it.
- Log structured objects via `host:log.info`.

---

## 14. Testing
- Unit: call handler blocks with mock `req`.
- Integration: spin dev server (`cleen serve`) and hit endpoints.
- E2E: verify SSR + hydration + API flows together.

---

## 15. CLI
```bash
cleen serve        # dev server with hot reload
cln compile main.cln -o app.wasm --plugins    # produce WASM
cleen api:spec     # OpenAPI
cleen api:sdk      # Client SDKs
```

---

## 16. Middleware

Middleware files live in `app/server/middleware/`. They inspect or modify requests before the endpoint handler runs.

```clean
// app/server/middleware/RateLimit.cln
middleware RateLimit
    functions:
        Request handle(Request req)
            string key = "rate:" + req.ip
            integer count = cache.get(key).toInteger()
            if count > 100
                return error(429, "Too many requests")
            cache.set(key, (count + 1).toString(), ttl: 60)
            return req
```

Register middleware globally in `main.cln`, or per-endpoint using the `middleware()` modifier:

```clean
endpoints:
    GET "/api/data" middleware(RateLimit, VerifyJWT) :
        return json({ ok: true })

    POST "/api/upload" [admin] middleware(RateLimit) :
        return json({ uploaded: true })
```

---

## 17. Routing File (`app/web/routes.cln`)

The `routes.cln` file handles routing concerns that cannot be expressed as page files: guards, redirects, rewrites, and custom error pages. Grammar: `frame-server.ebnf §10`.

**Rule:** if a page maps cleanly to a URL, use `app/web/pages/`. Use `routes.cln` only when a page file is not sufficient — guards on path patterns, permanent/temporary redirects, path rewrites with dynamic resolution, or custom error responses.

```clean
// app/web/routes.cln
routes:
    redirect: "/old-about" → "/about"
    guard: "/admin/*" [admin]
    error: 404 → "pages/not-found.cln"
    rewrite: "/:username" → "pages/profile/[id].cln"
        resolve: username → User.first({ handle: username })
```

**Directive reference**

| Directive | Purpose |
|-----------|---------|
| `redirect: <from> → <to>` | Permanent (301) redirect from one path to another |
| `guard: <pattern> [roles]` | Require at least one listed role to access the matched paths |
| `error: <code> → <page>` | Map an HTTP error status to a custom error page |
| `rewrite: <pattern> → <page>` | Rewrite a URL pattern to a page file; `resolve:` sub-block maps path segments to model lookups |

---

## 18. Server-Sent Events (STREAM Endpoints)

Use the `STREAM` method to declare a Server-Sent Event endpoint. The server responds with `Content-Type: text/event-stream` and keeps the connection open until the handler calls `sse.close()` or returns.

```clean
endpoints:
    STREAM "/api/generate/{jobId}" :
        string jobId = req.params.jobId
        GenerationJob job = generation.startJob(jobId)

        sse.emit(json.encode({ status: "started", job: jobId }))

        for task in job.tasks:
            if not sse.isConnected():
                generation.cancelJob(jobId)
                return

            generation.runTask(task)
            sse.emitEvent("task-complete", json.encode({
                taskId: task.id,
                label:  task.label,
                status: "done"
            }))

        sse.emitEvent("generation-complete", json.encode({ files: job.outputFiles }))
        sse.close()
```

**Rules:**
- `STREAM` routes are GET-only at the HTTP level. The plugin registers them via `_http_sse_route("GET", ...)`.
- No return type annotation is valid on a `STREAM` handler — it never produces a single response value.
- Inline modifiers (`[guard]`, `cache()`, `middleware()`) are supported in the same fixed order as regular routes.

### 18.1 SSE Functions

| Function | Description |
|----------|-------------|
| `sse.emit(data)` | Send `data: {payload}\n\n` to the client |
| `sse.emitEvent(name, data)` | Send `event: {name}\ndata: {payload}\n\n` to the client |
| `sse.close()` | Close the stream gracefully |
| `sse.retry(ms)` | Tell the client to reconnect after `ms` milliseconds if disconnected |
| `sse.isConnected() -> boolean` | Returns `false` when the client has disconnected; use to abort long loops |

### 18.2 Client-Side: `cl-stream` Directive

Connect an HTML element to a STREAM endpoint with `cl-stream`:

```html
<div
    class="task-list"
    cl-stream="/api/generate/{jobId}">
</div>
```

- On mount the browser opens an `EventSource` to the URL.
- Each `data:` event replaces the element's `innerHTML` with the payload.
- Named events (from `sse.emitEvent`) are dispatched as `CustomEvent` on the element.
- The connection closes when the server calls `sse.close()`, or when the element unmounts.
- The URL may include `{interpolated}` expressions from the component's state.

---

## 19. Server-Side HTTP Client  <!-- was §18 -->

The server can make outbound HTTP requests using the `http.*` bridge functions. This is separate from client-side communication (`frame.client`).

```clean
// app/server/api/proxy.cln
endpoints:
    GET "/api/weather":
        string city = req.query.city
        string url = "https://weather.example.com/api?city=" + city
        string response = http.get(url)
        return json(response)

    POST "/api/notify":
        string body = req.json(string)
        string result = http.postJson("https://notify.example.com/send", body)
        return json({ sent: true })
```

Available: `http.get(url)`, `http.postJson(url, body)`, `http.put(url, body)`, `http.delete(url)`.

> **Note:** These functions are a clean API layer over the internal `host:http` bridge — they execute on the server and make outbound network calls from the server process. They are NOT the same as `frame.client` (which provides browser-side HTTP calls from client-side WASM). Use `http.*` for server-to-server communication; use `frame.client` for browser-to-server communication.

---

## 20. Security
- Enforce HTTPS at host; set HSTS as appropriate.
- Use `HttpOnly` + `SameSite` cookies for sessions.
- Limit CORS; expose only required origins/headers.
- Validate DTOs on input and sanitize outputs.

---

## 21. Examples

### 20.1 Simple CRUD
```clean
// /app/server/api/posts.cln
endpoints:
    GET "/api/posts" :
        list<Post> posts = Post.find:
            order:
                createdAt desc
            limit: 20
        return json(posts)

    POST "/api/posts" :
        CreatePost body = req.json(CreatePost)
        Post p = Post.insert:
            title   = body.title
            content = body.content
            author  = body.author
        return json(p), status(201)
```

### 20.2 Guard + Cache + Middleware
```clean
endpoints:
    GET "/api/reports/daily" [admin] cache(300) :
        Report r = buildDailyReport()
        return json(r)

    GET "/api/reports/live" [admin, analyst] cache("no-store") middleware(Audit) :
        Report r = buildLiveReport()
        return json(r)
```

---

## 22. File Download Response (`res.download`)

Calling `res.download(filename)` sets `Content-Disposition: attachment; filename="<filename>"` on the response before it is sent. The browser presents a save-file dialog instead of rendering the content inline.

Call `res.download` before the `return` statement. It modifies the pending response headers; it does not send data itself.

```clean
endpoints:
    GET "/api/reports/daily":
        string pdf = reports.generateDailyPdf()
        res.download("daily-report.pdf")
        return res.binary(pdf, "application/pdf")

    GET "/api/data/export":
        string csv = db.exportCsv("SELECT * FROM orders")
        res.download("orders.csv")
        return res.text(csv, "text/csv")
```

---

## 23. File Upload (Multipart)

Multipart form uploads are accessed through `req.files()` and `req.file(fieldName)`.

- `req.files()` returns all uploaded files as `Array<UploadedFile>`.
- `req.file("fieldName")` returns the first file uploaded under that input name.
- Temporary files written to disk are cleaned up automatically after the handler returns.
- Validate MIME type and size before processing; do not trust the client-provided values without checking `mimeType` and `size`.

### `UploadedFile` Fields

| Field | Type | Description |
|---|---|---|
| `fieldName` | `string` | The `<input name>` attribute |
| `originalName` | `string` | The filename the user selected |
| `mimeType` | `string` | Detected MIME type |
| `size` | `integer` | File size in bytes |
| `tempPath` | `string` | Path to the temp file on server disk |
| `content` | `string` | File content as string (text files < 1 MB only) |

### Example

```clean
endpoints:
    POST "/api/avatar" [user]:
        UploadedFile avatar = req.file("avatar")
        if avatar.mimeType != "image/jpeg" and avatar.mimeType != "image/png":
            return badRequest("Only JPEG and PNG allowed")
        if avatar.size > 5242880:
            return badRequest("File too large")
        string path = files.saveUpload(avatar.tempPath, "avatars/" + req.userId)
        return json({ path: path })
```

---

## 24. Email Sending — SMTP Core

**Scope:** Transactional email only. No templates, no queuing, no multi-provider. Templates and queuing are Tier 2 and will be specified in a dedicated `frame.email` spec.

### Configuration

SMTP connection settings are declared in a `mail:` block in `main.cln`. All values support `$ENV_VAR` syntax for environment variable substitution.

```clean
server:
    port: 3000

mail:
    host     = $SMTP_HOST
    port     = 587
    secure   = true
    username = $SMTP_USER
    password = $SMTP_PASS
    from     = "no-reply@myapp.com"
```

### Functions

| Function | Signature | Description |
|---|---|---|
| `email.send` | `(to, subject, html, text) -> boolean` | Send using the configured `from` address |
| `email.sendFrom` | `(from, to, subject, html, text) -> boolean` | Override the `from` address per call |
| `email.lastError` | `() -> string` | Error message from the most recent failed send; `""` on success |

`email.send` and `email.sendFrom` return `false` on failure — they do not throw. Call `email.lastError()` to retrieve the reason after a failed send.

### Example — Magic Link

```clean
endpoints:
    POST "/auth/magic-link":
        string address = req.json("email")
        string token   = auth.createMagicLink(address, 3600)
        string link    = "https://myapp.com/auth/verify?token=" + token

        boolean sent = email.send(
            address,
            "Your sign-in link",
            "<p><a href=\"" + link + "\">Sign in</a></p>",
            "Sign in: " + link
        )

        if not sent:
            return status(500, json({ error: "Could not send login email" }))
        return json({ sent: true })
```

### Non-Blocking Send

Use the `background` keyword to dispatch the send without holding up the HTTP response:

```clean
background email.send(
    order.customerEmail,
    "Order confirmed",
    "<p>Your order #" + order.id + " is confirmed.</p>",
    "Your order #" + order.id + " is confirmed."
)
```

---

## 25. WebSocket Endpoints

WebSocket endpoints handle persistent bidirectional connections. They are declared with the `WEBSOCKET` method keyword inside an `endpoints:` block.

### 25.1 Syntax

```clean
endpoints:
    WEBSOCKET "/ws/chat" :
        // onConnect body — runs when a client establishes the connection
        ws.roomJoin(ws.clientId(), "lobby")

        ws.onMessage:
            // runs each time the client sends a frame
            string msg = ws.message()
            integer id = ws.clientId()
            ws.roomBroadcast("lobby", "{\"from\":" + id.toString() + ",\"text\":\"" + msg + "\"}")

        ws.onClose:
            // runs when the connection closes (client disconnect or server close)
            ws.roomLeave(ws.clientId(), "lobby")
```

### 25.2 Lifecycle

| Phase | Handler | Bridge function called |
|-------|---------|----------------------|
| Connection established | Route body (onConnect) | `_http_ws_route` registers it |
| Message received | `ws.onMessage:` sub-block | Called per frame |
| Connection closed | `ws.onClose:` sub-block | Called once on disconnect |

### 25.3 WebSocket Bridge Functions

All WebSocket bridge functions are server-only (Layer 3). They may only be called inside `WEBSOCKET` endpoint handlers.

| Function | Signature | Description |
|----------|-----------|-------------|
| `ws.clientId()` | `-> integer` | Integer ID of the client whose handler is executing |
| `ws.message()` | `-> string` | UTF-8 payload of the incoming frame (onMessage only) |
| `ws.send(clientId, message)` | `(integer, string)` | Send a text frame to a specific client |
| `ws.broadcast(room, message)` | `(string, string)` | Send a frame to all clients in a room |
| `ws.close(clientId)` | `(integer)` | Close a client's connection with normal closure (1000) |
| `ws.roomJoin(clientId, room)` | `(integer, string)` | Add a client to a named room |
| `ws.roomLeave(clientId, room)` | `(integer, string)` | Remove a client from a named room |
| `ws.roomBroadcast(room, message)` | `(string, string)` | Alias for `ws.broadcast` |

### 25.4 Auth Guards

Role guards are supported on `WEBSOCKET` routes:

```clean
endpoints:
    WEBSOCKET "/ws/admin" [admin] :
        ws.roomJoin(ws.clientId(), "admin-channel")
```

The guard is evaluated at connection time. Unauthorized connections receive a `401` HTTP response before the WebSocket upgrade and are never established.

### 25.5 Room System

Rooms are in-memory sets of client IDs managed by the server runtime. A client may be in multiple rooms simultaneously. Rooms are created when the first client joins and destroyed when the last client leaves.

### 25.6 Complete Example — Chat Room

```clean
// app/server/api/chat.cln
endpoints:
    WEBSOCKET "/ws/chat/:room" :
        string room = req.params.room
        integer cid = ws.clientId()
        ws.roomJoin(cid, room)
        ws.broadcast(room, "{\"event\":\"joined\",\"id\":" + cid.toString() + "}")

        ws.onMessage:
            string payload = ws.message()
            integer sender = ws.clientId()
            ws.roomBroadcast(req.params.room, "{\"event\":\"message\",\"from\":" + sender.toString() + ",\"text\":" + payload + "}")

        ws.onClose:
            integer cid = ws.clientId()
            ws.roomLeave(cid, req.params.room)
            ws.broadcast(req.params.room, "{\"event\":\"left\",\"id\":" + cid.toString() + "}")
```

---

**End of Document 03 — Frame Server Specification (endpoints, routing)**

