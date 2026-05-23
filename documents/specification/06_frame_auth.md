# Frame Auth Specification (06)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.0  
**Location:** `/documents/specification/06_frame_auth.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

## 1. Introduction

**Frame Auth** provides a unified authentication and authorization layer for applications built with **Clean Language** and the Frame framework. It covers **sessions**, **JWT**, **roles/permissions**, and **policy guards** that integrate with Server and UI seamlessly.

Auth is designed to be **simple by default** (cookies + sessions) and **extensible** (JWT, multi-tenant, SSO) using the **Host Bridge**.

---

## 2. Goals

| Goal | Description |
|------|-------------|
| **Secure by default** | Safe cookies, CSRF protection, and least-privilege rules. |
| **Type-safe** | Auth state and claims are fully typed in Clean. |
| **Simple to use** | Minimal config for common cases, readable APIs. |
| **Composable** | Guards integrate with routes, UI actions, and data. |
| **Extensible** | Support JWT, OAuth2/OIDC, magic links, and SSO via plugins. |

---

## 3. Key Concepts

- **Identity**: A verified user (id + claims).  
- **Session**: Server-managed identity stored in an HTTP-only cookie.  
- **Token (JWT)**: Self-contained identity for APIs or mobile/desktop apps.  
- **Role**: A named set of permissions (e.g., `admin`, `editor`, `viewer`).  
- **Permission**: A typed capability (e.g., `post.create`, `post.publish`).

---

## 4. Configuration

**File:** `/app/auth/auth.cln`
```clean
auth:
    session:
        cookie = "frame.sid"
        sameSite = "Lax"         // Lax | Strict | None
        secure = true            // use HTTPS-only cookies
        httpOnly = true          // inaccessible to JS
        domain? = null
        path = "/"
        timeoutMinutes = 60

    jwt:
        enabled = true
        secret = env("JWT_SECRET")
        alg = "HS256"
        ttlMinutes = 60
        refreshTtlMinutes = 43200   // 30 days

    roles:
        admin: ["*"]
        editor: ["post.create", "post.edit", "post.read"]
        viewer: ["post.read"]
```

**File:** `/app/auth/roles.cln` (optional override)
```clean
roles:
    admin: ["*"]
    editor: ["post.create", "post.edit", "post.read"]
    viewer: ["post.read"]
```

### Session Store

The default session store is **in-memory**, suitable for single-instance deployments (development, single-server production). For production multi-instance deployments (load-balanced or horizontally-scaled), a persistent session store must be configured via the session store adapter interface to ensure sessions are shared across all instances. Redis and database-backed session stores are the recommended production adapters; the configuration key for the adapter is `session.store` (not shown in the basic config above — omitting it selects the in-memory default).

### Password Hashing

Password hashing and verification use `host:crypto` functions (`hash` and `verify`). The implementation uses platform-native cryptography; the specific library is host-adapter dependent (commonly argon2 or bcrypt). Custom algorithms must not be used — see AUTH-C007.

---

## 5. Session Authentication (Cookie-based)

### Flow
1. User submits credentials to `/auth/login`.  
2. Server validates and creates a session (stored in the server store).  
3. Server sets `frame.sid` cookie (HTTP-only, Secure, SameSite).  
4. Subsequent requests attach the cookie automatically.

### Example: Login handler (Clean)
```clean
// app/server/api/auth.cln
endpoints:
    POST "/auth/login" :
        LoginForm form = req.json(LoginForm)
        User? u = User.first:
            where:
                email == form.email
        if u == null or not verifyPassword(form.password, u.hash)
            return badRequest("Invalid credentials")

        Session s = auth.session.create(u.id, claims: { email: u.email, role: u.role })
        return auth.session.setCookie(s, redirect("/dashboard"))

    POST "/auth/logout" :
        auth.session.destroyCurrent()
        return redirect("/login")
```

### Refresh Token Endpoint

```clean
endpoints:
    POST "/auth/refresh" :
        string? refreshToken = req.cookie("frame.refresh")
        if refreshToken == null
            return unauthorized()
        string newToken = auth.jwt.refresh(refreshToken)
        if newToken == ""
            return unauthorized()
        return json({ token: newToken })
```

### CSRF
- **POST/PUT/PATCH/DELETE** require a CSRF token.  
- Token is stored server-side with the session and included as a hidden input or header `X-CSRF-Token`.

---

## 6. JWT Authentication (API/Mobile/Desktop)

Use JWT for APIs, mobile/desktop apps (Capacitor/Tauri) or service-to-service calls.

### Issue Token
```clean
functions:
    Token issueToken(User u)
        return auth.jwt.sign({
            sub: u.id,
            email: u.email,
            role: u.role,
            iat: now(),
            exp: now().plusMinutes(auth.jwt.ttlMinutes)
        })
```

### Verify Token (middleware)
```clean
middleware VerifyJWT
    functions:
        Request handle(Request req)
            string? t = req.headers["Authorization"]
            if t == null or not t.startsWith("Bearer ")
                return error(401, "Missing token")

            Claims? c = auth.jwt.verify(t.substring(7))
            if c == null
                return error(401, "Invalid token")

            return req.withContext("claims", c)
```

### Refresh Flow
- Short-lived access tokens; long-lived refresh tokens (HTTP-only cookie or secure storage).  
- Endpoint `/auth/refresh` issues a new access token if the refresh token is valid.

**Refresh token policy — single use:** Refresh tokens are single-use. After a refresh token is used to obtain a new access token, it is immediately invalidated and a new refresh token is issued alongside the new access token. A second attempt to use the same refresh token returns an empty string (failure). This ensures that token theft is detectable: if an attacker reuses a refresh token that the legitimate client has already consumed, the server can detect the replay and revoke the session.

---

## 7. Roles & Permissions

### Check Permissions
```clean
if not auth.can(user, "post.publish")
    return error(403, "Forbidden")
```

### Guard in Endpoints
```clean
endpoints:
    POST "/admin/publish" [editor, admin]:
        return json({ ok: true })
```

### Policy Functions (Pro)
```clean
functions:
    boolean canPublish(User u)
        return u.role == "admin" or u.permissions.contains("post.publish")
```

**Resolution order:**
1) Explicit policy function on route (if present)  
2) Role-based guard  
3) Default deny

---

## 8. Multi‑Tenant (Optional)

Add `tenantId` to session claims and DB queries.
```clean
Session s = auth.session.create(u.id, claims: { tenantId: u.tenantId, role: u.role })
```
Applied to queries:
```clean
list<User> tenantUsers = User.find:
    where:
        tenantId == ctx.claims.tenantId
```
Database schemas or row‑level security can be used depending on scale.

---

## 9. UI Integration

- UI elements can be gated by `auth.can()` checks.  
- Hide actions or disable buttons when permissions are missing.

```html
<ui-button onClick="publish" disabled="{!auth.can(user, 'post.publish')}">Publish</ui-button>
```

- For client islands, claims are **not** exposed by default. Derive minimal booleans (e.g., `canPublish: true`) to keep secrets server-side.

---

## 10. Security Best Practices

- Use **HTTPS** always; set `secure=true` and `httpOnly=true` for cookies.
- Set `SameSite=Lax` or `Strict` for session cookies.
- Rotate **JWT secrets**; monitor key age.
- Prefer short token TTLs with refresh flow.
- Enforce **CSP**; avoid inline scripts if possible.
- Rate-limit login endpoints.
- Log suspicious activity (failed logins, token reuse) with `host:log`.

---

## 11. Host Bridge Integration

| Namespace | Functions | Notes |
|-----------|-----------|-------|
| `host:crypto` | `random`, `hash`, `verify` | Password hashing (argon2/bcrypt) |
| `host:env` | `get` | Secrets like `JWT_SECRET` |
| `host:log` | `info`, `warn`, `error` | Security logs |
| `host:http` | `setCookie`, `getCookie`, `redirect` | Cookie ops & redirects |
| `host:time` | `now`, `sleep` | Token TTL calculations |

**Error Envelope** (standard):
```json
{ "ok": false, "err": { "code": "AUTH_ERROR", "message": "..." } }
```

---

## 12. Error Codes (Suggested)

| Code | Meaning |
|------|---------|
| `INVALID_CREDENTIALS` | Wrong email/password |
| `SESSION_EXPIRED` | Session timed out |
| `TOKEN_INVALID` | Bad or expired JWT |
| `TOKEN_REPLAY` | Same token seen twice |
| `FORBIDDEN` | Missing permission |
| `CSRF_FAIL` | CSRF token mismatch |

---

## 13. Testing

- **Unit:** verify `auth.can()` and policy functions.
- **Integration:** login/logout, cookie set/clear, CSRF checks.
- **Security:** replay token attempts, brute-force detection.

Test helpers live in `/tests/auth/`.

---

## 14. AI Development Context

This file provides deterministic structures for AI agents:
- Typed claims and roles (consistent naming).
- Standard error envelope and codes.
- Clear bridge functions so agents can mock login flows.

When prompting an AI agent:
- Include this file + `03_frame_server.md` for route handling.
- Provide `/app/auth/auth.cln` and `/app/auth/roles.cln` snippets if relevant.

---

## 15. File Locations

- Config: `/app/auth/auth.cln`, `/app/auth/roles.cln`
- Server handlers: `/app/server/api/auth/*.cln`
- UI: gated components under `/app/ui/components/` and pages in `/app/ui/pages/`
- Tests: `/tests/auth/`

---

**End of Document 06**

