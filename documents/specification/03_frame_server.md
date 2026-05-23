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
/app/server/services/    # Business logic services
/app/server/middleware/  # Custom middleware
/app/ui/pages/*.html     # SSR page templates (HTML)
/app/ui/pages/*.cln      # Companion data loaders (paired by filename)
/app/ui/components/*.cln # UI components
/app/ui/layouts/*.html   # Page layout wrappers
/app/data/*.cln          # Data models / ORM
/app/auth/*.cln          # Auth configuration (frame.auth)
/public/*                # Static assets
```

**Plugin Folder Ownership:** Files placed in `app/server/`, `app/server/api/`, or `app/server/services/` are processed by the `frame.server` plugin. The plugin must be declared in `app.cln` via the `plugins:` block. Once declared, individual source files in plugin-owned folders do not need their own `import` statement — the folder location determines which plugin processes them.

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
- Server renders pages from `/app/ui/pages/*.html`. Data is supplied by paired companion `.cln` files.
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
cln compile app.cln -o app.wasm --plugins    # produce WASM
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

Register middleware globally in `app.cln`, or per-endpoint using the `middleware()` modifier:

```clean
endpoints:
    GET "/api/data" middleware(RateLimit, VerifyJWT) :
        return json({ ok: true })

    POST "/api/upload" [admin] middleware(RateLimit) :
        return json({ uploaded: true })
```

---

## 17. Server-Side HTTP Client

The server can make outbound HTTP requests using the `http.*` bridge functions. This is separate from client-side communication (`frame.client`).

```clean
// app/backend/api/proxy.cln
endpoints:
    GET "/api/weather":
        string city = req.query("city")
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

## 18. Security
- Enforce HTTPS at host; set HSTS as appropriate.
- Use `HttpOnly` + `SameSite` cookies for sessions.
- Limit CORS; expose only required origins/headers.
- Validate DTOs on input and sanitize outputs.

---

## 19. Examples

### 19.1 Simple CRUD
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

### 19.2 Guard + Cache + Middleware
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

**End of Document 03 — Frame Server Specification (endpoints)**

