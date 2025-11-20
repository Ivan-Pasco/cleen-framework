
1. Envelope Shape

Every message between Clean → Host or Host → Clean uses this JSON format:

{ "fn": "host:<namespace>.<function>", "args": { ... } }


Host responses must always return one of the following:

✅ Success

{ "ok": true, "data": { ... } }


❌ Error

{ "ok": false, "err": { "code": "ERROR_CODE", "message": "Readable description", "details": {} } }


Error codes are predictable (e.g. DB_ERROR, NETWORK_FAIL, AUTH_ERROR, etc.).

2. HTTP Bridge (host:http)

Used for outbound HTTP requests or direct host responses in server mode.

Request Example
{
  "fn": "host:http.request",
  "args": {
    "method": "GET",
    "url": "https://api.example.com/data",
    "headers": { "Accept": "application/json" },
    "body": null,
    "timeout": 5000
  }
}

Response
{
  "ok": true,
  "data": {
    "status": 200,
    "headers": { "content-type": "application/json" },
    "body": "{ \"msg\": \"OK\" }"
  }
}

Host Response Shortcut (for SSR)
{
  "fn": "host:http.respond",
  "args": { "status": 302, "headers": { "Location": "/login" }, "body": "" }
}

3. Database Bridge (host:db)

Manages SQL execution and transactional operations.

Query
{
  "fn": "host:db.query",
  "args": { "sql": "SELECT * FROM users WHERE id=$1", "params": [1] }
}

Transaction
{
  "fn": "host:db.tx",
  "args": {
    "ops": [
      { "sql": "INSERT INTO users (name) VALUES ($1)", "params": ["Ana"] },
      { "sql": "INSERT INTO posts (title, user_id) VALUES ($1, $2)", "params": ["Hello", 1] }
    ]
  }
}

Configuration
{
  "fn": "host:db.config",
  "args": { "driver": "postgres", "pool": { "max": 10, "idleTimeout": 30000 } }
}

Example Error
{
  "ok": false,
  "err": { "code": "DB_ERROR", "message": "unique violation on users.email", "details": { "constraint": "users_email_key" } }
}

4. Environment Bridge (host:env)

Used to read secrets or environment variables from the host.

Get Value
{
  "fn": "host:env.get",
  "args": { "name": "JWT_SECRET" }
}

List All
{
  "fn": "host:env.list",
  "args": {}
}


Response

{ "ok": true, "data": { "JWT_SECRET": "xyz", "PORT": "8080" } }

5. Time Bridge (host:time)

Provides access to time utilities.

Current Time
{ "fn": "host:time.now" }


Response:

{ "ok": true, "data": { "iso": "2025-11-04T20:00:00Z", "epoch": 1730740800 } }

Sleep
{ "fn": "host:time.sleep", "args": { "ms": 500 } }

6. Crypto Bridge (host:crypto)

Handles randomness, hashing, signing, and verification.

Random Bytes
{
  "fn": "host:crypto.random",
  "args": { "bytes": 32 }
}


Response:

{ "ok": true, "data": { "base64": "Ejd92jshf82jsf8..." } }

Hash
{
  "fn": "host:crypto.hash",
  "args": { "algo": "sha256", "data": "base64:SGVsbG8=" }
}

Verify Password
{
  "fn": "host:crypto.verify",
  "args": { "algo": "bcrypt", "data": "plaintext", "hash": "stored-hash" }
}

Sign / Verify JWT (optional adapter)
{
  "fn": "host:crypto.sign",
  "args": { "data": { "sub": 1 }, "secret": "key", "alg": "HS256" }
}

7. Log Bridge (host:log)

Writes logs to the host system.

Examples
{ "fn": "host:log.info", "args": { "event": "server.start", "port": 8080 } }
{ "fn": "host:log.warn", "args": { "event": "auth.failed", "user": "ana@x.com" } }
{ "fn": "host:log.error", "args": { "event": "db.error", "message": "unique violation" } }


Host must timestamp and append contextual metadata automatically.

8. Filesystem Bridge (host:fs)

May be disabled in some hosts (like browsers).
Provides limited sandboxed FS operations for build tools or desktop apps.

Read File
{ "fn": "host:fs.read", "args": { "path": "public/ui.css" } }


Response:

{ "ok": true, "data": { "bytes": "base64:..." } }

Write File
{ "fn": "host:fs.write", "args": { "path": "public/out.txt", "data": "base64:SGVsbG8=" } }

List Directory
{ "fn": "host:fs.list", "args": { "path": "public/" } }

9. System Bridge (host:sys)

For system-level operations (used only in CLI/Server environments).

Exit
{ "fn": "host:sys.exit", "args": { "code": 0 } }

Get Platform Info
{ "fn": "host:sys.platform" }


Response:

{ "ok": true, "data": { "os": "linux", "arch": "x86_64", "runtime": "node20" } }

10. Error Format (Unified)

Example envelope:

{
  "ok": false,
  "err": {
    "code": "NETWORK_FAIL",
    "message": "Unable to reach host",
    "details": { "url": "https://api.example.com" }
  }
}


Common error codes

Code	Meaning
DB_ERROR	Database or transaction failure
AUTH_ERROR	Unauthorized / invalid credentials
NETWORK_FAIL	Connection or timeout error
VALIDATION_ERROR	Field or data constraint violation
FILE_NOT_FOUND	FS access error
PERMISSION_DENIED	Missing Host Bridge permission
NOT_FOUND	Resource not found
TIMEOUT	Operation exceeded allowed time
11. AI Development Notes

All bridge functions are stateless and idempotent where possible.

JSON shape is deterministic — AI tools can safely build, simulate, and validate calls.

During generation, always produce the "fn" and "args" properties explicitly.

Nested bridges (e.g. host:db.tx) must flatten operations into ops arrays for parsing simplicity.

Always return Error Envelopes; never throw raw host errors.

