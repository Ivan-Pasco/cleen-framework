# Frame Server Specification (03)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.2 (adopts `endpoints:` for HTTP APIs)  
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

## 5. Optional Sub‑blocks (Declarative)
Sub‑blocks improve clarity and documentation. All are optional.

```clean
endpoints:
    GET "/api/secure" :
        guard:
            role in ["admin", "editor"]
        returns:
            json list<User>
        cache:
            maxAge = 60
        handle:
            list<User> users = User.find:
                where:
                    active == true
            return json(users)
```

- **guard:** authorization rules (evaluated before handler).  
- **returns:** output declaration to drive OpenAPI/SDKs.  
- **cache:** HTTP cache hints for host adapters.  
- **handle:** If present, the logic lives inside; otherwise, the endpoint body is the handler.

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
Auth lives in `app/auth/auth.cln` and exposes guards usable inside `guard:` blocks or imperative checks.

```clean
endpoints:
    GET "/api/admin" :
        guard:
            role in ["admin"]
        handle:
            return json({ ok: true })
```

You can also check explicitly:
```clean
if not auth.can(user, "post.publish"):
    return unauthorized()
```

---

## 10. OpenAPI & SDK Generation
The server can emit OpenAPI 3.1 by inspecting `endpoints:` declarations, `returns:`, typed params, and DTOs.

```bash
cleen api:spec   # writes openapi.json
cleen api:sdk    # generates Clean/TS/Swift/Kotlin clients
```

**Conventions**
- If `returns:` is omitted, the tool infers from `return` calls.
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

Register middleware in `app.cln` or per-endpoint:

```clean
endpoints:
    GET "/api/data" :
        middleware: [RateLimit, VerifyJWT]
        handle:
            return json({ ok: true })
```

---

## 17. Server-Side HTTP Client

The server can make outbound HTTP requests using the `http.*` bridge functions. This is separate from client-side communication (`frame.client`).

```clean
// app/server/api/proxy.cln
endpoints:
    GET "/api/weather" :
        handle:
            string city = req.query.city
            string url = "https://weather.example.com/api?city=" + city
            string response = http.get(url)
            return json(response)

    POST "/api/notify" :
        handle:
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

### 19.2 Guard + Returns + Cache
```clean
endpoints:
    GET "/api/reports/daily" :
        guard:
            role in ["admin"]
        returns:
            json Report
        cache:
            maxAge = 300
        handle:
            Report r = buildDailyReport()
            return json(r)
```

---

**End of Document 03 — Frame Server Specification (endpoints)**

