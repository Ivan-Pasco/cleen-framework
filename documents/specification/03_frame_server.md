# Frame Server Specification (03)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.2 (adopts `endpoints:` for HTTP APIs)  
**Location (repo):** `/docs/specification/03_frame_server.md`

---

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
```
/app/api/*.cln           # API modules using `endpoints:`
/app/pages/*.html        # SSR page templates (HTML)
/app/pages/*.cln         # Companion data loaders (paired by filename)
/app/components/*.cln    # UI components
/app/layouts/*.html      # Page layout wrappers
/app/data/*.cln          # Data models / ORM
/app/auth/*.cln          # Auth configuration
/public/*                # Static assets
```

---

## 4. Declaring HTTP APIs with `endpoints:`
Each API module exposes a single `endpoints:` block. Endpoints are declared by **METHOD + PATH** and the block body handles the request.

```clean
# /app/api/users.cln
endpoints:
    GET /api/users:
        list<User> users = User.find:
            where:
                active == true
            order:
                createdAt desc
            limit: 50
        return json(users)

    POST /api/users:
        CreateUser body = req.json(CreateUser)
        User u = User.insert:
            name  = body.name
            email = body.email
            active = true
        return json(u), status(201)

    GET /api/users/:id:
        integer id = req.params.id
        User? u = User.first:
            where:
                id == id
        if u == null:
            return notFound()
        return json(u)
```

**Notes**
- The body of `METHOD /path:` is the handler. No extra `functions:` wrapper is required.
- The handler must `return` a Response (e.g., `json(...)`, `html(...)`, `redirect(...)`).

---

## 5. Optional Sub‑blocks (Declarative)
Sub‑blocks improve clarity and documentation. All are optional.

```clean
endpoints:
    GET /api/secure:
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
Auth lives in `/config/auth.cln` and exposes guards usable inside `guard:` blocks or imperative checks.

```clean
endpoints:
    GET /api/admin:
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
frame api:spec   # writes openapi.json
frame api:sdk    # generates Clean/TS/Swift/Kotlin clients
```

**Conventions**
- If `returns:` is omitted, the tool infers from `return` calls.
- Path params (`:id`) are typed from local variable assignments (e.g., `integer id = req.params.id`).

---

## 11. SSR Pipeline (UI)
- Server renders pages from `/app/pages/*.html`. Data is supplied by paired companion `.cln` files.
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
- Integration: spin dev server (`frame serve`) and hit endpoints.
- E2E: verify SSR + hydration + API flows together.

---

## 15. CLI
```bash
frame serve        # dev server with hot reload
frame build        # produce optimized WASM + assets
frame api:spec     # OpenAPI
frame api:sdk      # Client SDKs
```

---

## 16. Security
- Enforce HTTPS at host; set HSTS as appropriate.
- Use `HttpOnly` + `SameSite` cookies for sessions.
- Limit CORS; expose only required origins/headers.
- Validate DTOs on input and sanitize outputs.

---

## 17. Examples

### 17.1 Simple CRUD
```clean
# /app/api/posts.cln
endpoints:
    GET /api/posts:
        list<Post> posts = Post.find:
            order:
                createdAt desc
            limit: 20
        return json(posts)

    POST /api/posts:
        CreatePost body = req.json(CreatePost)
        Post p = Post.insert:
            title   = body.title
            content = body.content
            author  = body.author
        return json(p), status(201)
```

### 17.2 Guard + Returns + Cache
```clean
endpoints:
    GET /api/reports/daily:
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

