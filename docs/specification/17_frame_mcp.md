# Frame MCP Specification (17)

**Project:** Frame – Full-Stack Framework for Clean Language
**Version:** 1.0.0
**Location (repo):** `/documents/specification/17_frame_mcp.md`

---

> **See also:** [Architecture Boundaries](../../../foundation/management/ARCHITECTURE_BOUNDARIES.md) — component responsibilities and cross-component work policy.

## 1. Purpose

**frame.mcp** is the plugin for building **Model Context Protocol (MCP) servers** in Clean Language. MCP is an open protocol (built on JSON-RPC 2.0) that standardizes how AI agents communicate with external tools, data sources, and prompt templates. An MCP server exposes three kinds of primitives to any compliant AI client (Claude, OpenAI, Cursor, etc.):

- **Tools** — callable functions the AI can invoke to take actions or retrieve computed data
- **Resources** — read-only URIs the AI can access to retrieve structured data
- **Prompts** — parameterized message templates the AI can render and inject into conversations

frame.mcp lets developers declare these primitives in Clean Language using a single declarative `mcp:` block per file. The plugin owns the `app/mcp/` folder, handles the complete MCP lifecycle (capability negotiation, method dispatch, error formatting), and provides two transports: `stdio` for local process-based servers and `http` for remote/cloud-hosted servers using HTTP + Server-Sent Events (SSE). Developers write only the business logic — the protocol machinery is invisible.

**Goals**
- One obvious way to expose AI-callable tools, resources, and prompts.
- Full MCP protocol compliance without any JSON-RPC boilerplate.
- Composable with frame.data for database-backed tools and resources.
- Transport-agnostic: switch between stdio and HTTP with one config line.

---

## 2. Responsibilities

**frame.mcp owns:**
- Processing `app/mcp/*.cln` files via the `mcp:` block
- Transport lifecycle: stdio read/write loop and HTTP+SSE server
- MCP initialize handshake and capability negotiation
- JSON-RPC 2.0 method dispatch: `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, `prompts/get`
- Serialization of tool params to typed Clean variables
- Serialization of Clean return values to MCP content blocks
- Error normalization: unhandled exceptions → MCP error results
- API key authentication guard (when configured)

**frame.mcp does NOT own:**
- Database queries — delegated to frame.data via ORM calls in tool bodies
- Session/cookie authentication — MCP clients are AI agents, not browsers; use `apiKey:` if auth is needed
- HTTP routing for application endpoints — use frame.server for that
- Client-side communication — use frame.client for browser-to-server calls

---

## 3. File Layout

> **Canonical reference:** [PROJECT_STRUCTURE.md](../PROJECT_STRUCTURE.md) — complete folder reference.

```
/app/mcp/                    # Owned by frame.mcp plugin
/app/mcp/*.cln               # MCP server files using the mcp: block
/app/mcp/blog-tools.cln      # Example: blog tool definitions
/app/mcp/user-tools.cln      # Example: user management tools
```

**Plugin Folder Ownership:** Files placed in `app/mcp/` are processed exclusively by the `frame.mcp` plugin. The plugin must be declared in `main.cln` via the `target:` block. Once declared, all `.cln` files in `app/mcp/` are automatically processed — no per-file import statements are needed (`implicit_import = true`).

Multiple files in `app/mcp/` are each compiled independently. Each file produces its own `mcp:` server with its own name, tool set, and transport configuration. To bundle tools across multiple logical groupings into a single server, place them all in one file under a single `mcp:` block.

---

## 4. The `mcp:` Block

Each `app/mcp/*.cln` file contains exactly one `mcp:` block. It is the root container for all tool, resource, and prompt definitions.

### 4.1 Syntax

```clean
mcp "server-name":
    version: "1.0.0"
    description: "Human-readable description for AI clients"
    transport: stdio
```

For HTTP transport:

```clean
mcp "server-name":
    version: "1.0.0"
    description: "Human-readable description for AI clients"
    transport: http:
        port: 3001
        host: "0.0.0.0"
```

With API key authentication:

```clean
mcp "secure-server":
    version: "1.0.0"
    description: "Authenticated MCP server"
    transport: http:
        port: 3001
        host: "localhost"
    apiKey: env.get("MCP_API_KEY")
```

### 4.2 Top-Level Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `version` | string | no | `"1.0.0"` | Server version string reported during initialize handshake |
| `description` | string | no | `""` | Human-readable description sent in capability negotiation |
| `transport` | keyword | yes | — | Transport mode: `stdio` or `http: port: N host: "..."` |
| `apiKey` | string expression | no | — | When set, all incoming requests must supply this value in the `Authorization: Bearer <key>` header (HTTP transport) or in an `x-api-key` field in the JSON-RPC params meta (stdio transport) |

### 4.3 One `mcp:` Block Per File Rule

Each `.cln` file in `app/mcp/` may contain exactly one `mcp:` block. Declaring two `mcp:` blocks in the same file is a compile error. Sub-blocks (`tool`, `resource`, `prompt`) all live inside the single `mcp:` block body.

### 4.4 main.cln Registration

Declare frame.mcp in the `target:` block alongside any other plugins the project uses:

```clean
// main.cln
target:
    platform: server
    plugins:
        frame.mcp
        frame.data
        frame.server
```

Only the plugins listed in `target:` are active. Files in `app/mcp/` are ignored unless `frame.mcp` is declared.

---

## 5. Tools

A `tool` sub-block declares a callable function the AI can invoke. When the MCP client sends a `tools/call` request, the plugin deserializes the params, runs the tool body, and returns the result as a content block.

### 5.1 Syntax

```clean
mcp "blog-tools":
    transport: stdio

    tool "list_posts":
        description: "List published blog posts with optional pagination"
        params:
            limit: integer = 10
            offset: integer = 0
        list<Post> posts = Post.find:
            where:
                published == true
            order:
                createdAt desc
            limit: limit
            offset: offset
        return json(posts)
```

### 5.2 `params:` Block

The `params:` block declares the inputs the tool accepts. Each parameter is declared as:

```
<name>: <type>
<name>: <type> = <default>
```

Parameters with a default value are **optional** — the AI may omit them and the default is used. Parameters without a default are **required** — the AI must supply them. Required parameters must be declared before optional ones.

```clean
tool "get_post":
    description: "Get a single post by ID"
    params:
        id: integer
        includeComments: boolean = false
    Post? post = Post.first:
        where:
            id == params.id
    if post == null:
        return error("Post not found")
    return json(post)
```

### 5.3 Accessing Parameters

Inside a tool body, parameters are accessed via the `params.*` accessor:

```clean
tool "search_posts":
    description: "Search posts by keyword"
    params:
        query: string
        limit: integer = 20
    list<Post> results = Post.find:
        where:
            title contains params.query
        limit: params.limit
    return json(results)
```

### 5.4 Parameter Type Rules

Valid types in `params:` correspond to JSON Schema types used in the tool's `inputSchema`. See Section 9 for the full mapping. The most common types are:

| Clean type | Accepted JSON value |
|-----------|---------------------|
| `string` | JSON string |
| `integer` | JSON integer |
| `number` | JSON number (float or integer) |
| `boolean` | JSON boolean |
| `list<string>` | JSON array of strings |
| `any` | Any JSON value |

### 5.5 Tool Body

The tool body is ordinary Clean Language code. All language features are available: variables, conditionals, loops, ORM queries via frame.data, and function calls. The body must end with a `return` statement using one of the response helpers defined in Section 10.

```clean
tool "create_post":
    description: "Create a new blog post in draft state"
    params:
        title: string
        content: string
        authorId: integer
    Post p = Post.insert:
        title = params.title
        content = params.content
        authorId = params.authorId
        published = false
        createdAt = time.now()
    return json(p)
```

### 5.6 Required vs Optional Parameters — Full Example

```clean
tool "update_post":
    description: "Update a post. Only supply the fields you want to change."
    params:
        id: integer
        title: string = ""
        content: string = ""
        published: boolean = false
    Post? post = Post.first:
        where:
            id == params.id
    if post == null:
        return error("Post not found", code: "NOT_FOUND")
    Post updated = Post.update:
        where:
            id == params.id
        title = params.title
        content = params.content
        published = params.published
    return json(updated)
```

---

## 6. Resources

A `resource` sub-block declares a read-only URI endpoint the AI can read. Resources are best suited for data that does not change during a single AI session, or data the AI needs to understand context (schema definitions, configuration, recent summaries).

### 6.1 Syntax

```clean
mcp "blog-tools":
    transport: stdio

    resource "posts://recent":
        description: "The 10 most recently published posts"
        mimeType: "application/json"
        list<Post> posts = Post.find:
            where:
                published == true
            order:
                createdAt desc
            limit: 10
        return json(posts)
```

### 6.2 Resource URI Format

The resource name is the URI used in `resources/read` requests. URIs follow a `scheme://path` format. Common patterns:

| Pattern | Example | Meaning |
|---------|---------|---------|
| `scheme://noun` | `posts://recent` | Well-known dataset |
| `scheme://noun/sub` | `config://database` | Named sub-configuration |
| `scheme://collection` | `users://active` | Filtered collection |

URI names must be valid URI strings (no spaces). They are matched exactly — there is no wildcard routing in resources.

### 6.3 `mimeType:` Field

| Value | When to use |
|-------|-------------|
| `"application/json"` | Structured data (default for ORM results) |
| `"text/plain"` | Unstructured text content |
| `"text/markdown"` | Markdown documents |
| `"text/csv"` | Tabular data for AI analysis |
| `"application/xml"` | XML payloads |

`mimeType:` is optional. If omitted, the plugin infers `"application/json"` when `json(value)` is returned, and `"text/plain"` when `text(str)` is returned.

### 6.4 Resource Body

The resource body is ordinary Clean Language code with the same ORM and helper access as tools. Resources take no parameters — they are read-only, parameterless endpoints. If you need parameterized access, use a `tool` instead.

```clean
resource "config://settings":
    description: "Current site configuration"
    mimeType: "application/json"
    SiteConfig config = SiteConfig.first()
    if config == null:
        return error("No site configuration found")
    return json(config)

resource "users://count":
    description: "Total number of registered users"
    mimeType: "text/plain"
    integer count = User.count()
    return text(count.toString())
```

---

## 7. Prompts

A `prompt` sub-block declares a parameterized message template the AI can render. When the MCP client sends a `prompts/get` request, the plugin substitutes arguments into the template and returns the rendered string as a user-role message.

### 7.1 Syntax

```clean
mcp "blog-tools":
    transport: stdio

    prompt "summarize_post":
        description: "Generate a social media summary for a blog post"
        args:
            post_id: integer
            platform: string = "twitter"
        Post? post = Post.first:
            where:
                id == args.post_id
        if post == null:
            return error("Post not found")
        return "Summarize this post for {{ platform }}:\n\nTitle: {{ post.title }}\n\nContent: {{ post.content }}"
```

### 7.2 `args:` Block

The `args:` block follows the same syntax as `params:` in tools: `<name>: <type>` for required args and `<name>: <type> = <default>` for optional args. Arguments are accessed via the `args.*` accessor inside the prompt body.

### 7.3 Prompt Body

The prompt body may contain arbitrary Clean Language code to look up data before rendering the template. The body must end with a `return` statement that returns a string. The string may contain `{{ expression }}` interpolation syntax, which the plugin evaluates before sending the rendered message to the MCP client.

```clean
prompt "draft_reply":
    description: "Draft a reply to a user comment"
    args:
        comment_id: integer
        tone: string = "professional"
    Comment? comment = Comment.first:
        where:
            id == args.comment_id
    if comment == null:
        return error("Comment not found")
    return "Write a {{ tone }} reply to this comment:\n\n{{ comment.body }}\n\nAuthor: {{ comment.authorName }}"
```

### 7.4 Prompt Return Value

A prompt body must return a string (the prompt text). The MCP protocol wraps this in a `messages` array with `role: "user"`. Developers do not construct the JSON envelope — only the string content is returned.

---

## 8. Transport Configuration

### 8.1 stdio Transport

The `stdio` transport is the default and most common for local MCP servers. The plugin reads newline-delimited JSON-RPC 2.0 messages from `stdin`, writes responses to `stdout`, and routes all log output to `stderr` so it never pollutes the transport stream.

```clean
mcp "my-server":
    transport: stdio
```

**Stdio lifecycle:**
1. Process starts.
2. Plugin reads `initialize` request from stdin.
3. Plugin writes `initialize` response with declared capabilities to stdout.
4. Plugin enters read loop: reads requests, dispatches, writes responses.
5. Process exits when stdin closes (MCP client disconnects).

**Stdio log safety:** All calls to `mcp.log()` (and all framework-level errors) are written to stderr. Never write to stdout except via the plugin's JSON-RPC response machinery — doing so corrupts the transport.

### 8.2 HTTP Transport

The `http` transport listens for HTTP POST requests containing JSON-RPC 2.0 bodies. Server-Sent Events (SSE) are used for server-initiated notifications. This transport is suitable for remote or cloud-hosted MCP servers accessible to multiple clients.

```clean
mcp "my-server":
    transport: http:
        port: 3001
        host: "0.0.0.0"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `port` | integer | `3001` | TCP port the HTTP server listens on |
| `host` | string | `"localhost"` | Interface to bind. `"0.0.0.0"` accepts all interfaces. |

**HTTP transport protocol:**
- All JSON-RPC requests arrive as HTTP POST to `/` with `Content-Type: application/json`
- Responses are standard HTTP 200 with `Content-Type: application/json`
- Server-initiated notifications use the `/sse` endpoint (SSE stream)
- The plugin manages connection IDs for SSE clients internally

**HTTP transport with API key:**

```clean
mcp "secure-server":
    transport: http:
        port: 3001
        host: "0.0.0.0"
    apiKey: env.get("MCP_API_KEY")
```

When `apiKey:` is set, every HTTP request must include `Authorization: Bearer <key>`. Requests without a valid key receive a `401 Unauthorized` response. The API key is read from the environment at startup — it is never hardcoded.

### 8.3 Choosing a Transport

| Use case | Transport |
|----------|-----------|
| Claude Desktop, local tools, Cursor extensions | `stdio` |
| Remote servers, multi-user setups, cloud deployments | `http` |
| CI pipelines, one-shot tool invocations | `stdio` |
| Shared team MCP server behind a firewall | `http` with `apiKey:` |

---

## 9. Type Mapping

The MCP protocol requires each tool to declare an `inputSchema` (JSON Schema) describing its parameters. frame.mcp generates this schema automatically from the `params:` block. The mapping is:

| Clean type | JSON Schema |
|-----------|-------------|
| `string` | `{"type": "string"}` |
| `integer` | `{"type": "integer"}` |
| `number` | `{"type": "number"}` |
| `boolean` | `{"type": "boolean"}` |
| `list<string>` | `{"type": "array", "items": {"type": "string"}}` |
| `list<integer>` | `{"type": "array", "items": {"type": "integer"}}` |
| `list<number>` | `{"type": "array", "items": {"type": "number"}}` |
| `list<boolean>` | `{"type": "array", "items": {"type": "boolean"}}` |
| `list<any>` | `{"type": "array"}` |
| `any` | `{}` (no constraint — accepts any JSON value) |
| Named model type (e.g. `Post`) | `{"type": "object", "properties": {...}}` (auto-derived from frame.data model field declarations) |

Parameters declared without a default are added to the `required` array in the generated schema. Parameters with a default are omitted from `required`.

**Example — generated inputSchema for `get_post`:**

```json
{
  "type": "object",
  "properties": {
    "id": { "type": "integer" },
    "includeComments": { "type": "boolean" }
  },
  "required": ["id"]
}
```

---

## 10. Response Helpers

The following functions are available inside `tool`, `resource`, and `prompt` bodies. They are provided by the frame.mcp runtime and do not require an import.

| Function | Returns | Description |
|----------|---------|-------------|
| `json(value)` | MCP text content block | Serialize any Clean value (model, list, integer, string, boolean) to a JSON string and wrap in an MCP text content block with `type: "text"` |
| `text(str)` | MCP text content block | Return a plain text string as an MCP text content block |
| `error(message)` | MCP error result | Return an MCP error result with `isError: true` and the given message |
| `error(message, code: string)` | MCP error result | Return an MCP error result with `isError: true`, the given message, and a machine-readable error code |

**`json()` behavior:** Serializes the value to a compact JSON string. Model types are serialized using their declared fields. Lists are serialized as JSON arrays. Primitives are serialized directly.

**`error()` behavior:** Returns an MCP tool result with `isError: true`. The AI client receives this as a failed invocation and may surface the message to the user or retry. Using `error()` does not crash the server — the server continues accepting requests.

```clean
tool "delete_post":
    description: "Delete a post by ID. Returns confirmation on success."
    params:
        id: integer
    Post? post = Post.first:
        where:
            id == params.id
    if post == null:
        return error("Post not found", code: "NOT_FOUND")
    Post.delete:
        where:
            id == params.id
    return text("Post " + params.id.toString() + " deleted successfully")
```

---

## 11. Error Handling

### 11.1 `onError:` Syntax

Use the standard Clean Language `onError:` syntax to catch exceptions from individual statements inside a tool, resource, or prompt body:

```clean
tool "risky_operation":
    description: "Attempt an external operation"
    params:
        targetId: integer
    ExternalRecord record = ExternalService.fetch(params.targetId):
        onError:
            return error("External service unavailable. Try again later.", code: "SERVICE_UNAVAILABLE")
    return json(record)
```

`onError:` attaches to the preceding statement. If that statement throws, control jumps to the `onError:` block. Execution does not continue at the statement after `onError:` — the `onError:` block must return.

### 11.2 Unhandled Exceptions

If an exception propagates out of a tool, resource, or prompt body without being caught by `onError:`, frame.mcp intercepts it and returns an MCP error result:

```json
{
  "content": [{ "type": "text", "text": "Internal error: <exception message>" }],
  "isError": true
}
```

The server continues running — a single tool failure does not terminate the process.

### 11.3 JSON-RPC Protocol Errors

Protocol-level errors (invalid JSON, unknown method, malformed params) are handled by the plugin runtime and returned as standard JSON-RPC error objects. Developers do not handle these — they are invisible at the Clean language level.

| Scenario | JSON-RPC error code | When it occurs |
|----------|--------------------|-|
| Unknown method | `-32601` | Client calls a method not in the MCP spec |
| Invalid params | `-32602` | Required tool param missing or wrong type |
| Parse error | `-32700` | Malformed JSON in the request |
| Internal error | `-32603` | Unhandled exception in plugin runtime itself |

---

## 12. MCP Lifecycle

Developers do not implement any of the protocol lifecycle. frame.mcp handles everything automatically.

### 12.1 Initialize Handshake

When a client connects, it sends an `initialize` request. The plugin responds with the server's name, version, and a capabilities object derived from what is declared in the `mcp:` block:

```json
{
  "capabilities": {
    "tools": { "listChanged": false },
    "resources": { "subscribe": false, "listChanged": false },
    "prompts": { "listChanged": false }
  }
}
```

- `tools` capability is advertised if at least one `tool` block is declared.
- `resources` capability is advertised if at least one `resource` block is declared.
- `prompts` capability is advertised if at least one `prompt` block is declared.

If the `mcp:` block has no `tool` sub-blocks, the `tools` capability is omitted from the response. The same applies to `resources` and `prompts`.

### 12.2 Method Dispatch

After initialization, the plugin handles these methods automatically:

| Method | What the plugin does |
|--------|----------------------|
| `tools/list` | Returns all declared `tool` blocks with their name, description, and generated `inputSchema` |
| `tools/call` | Deserializes params, runs the matching tool body, returns the result |
| `resources/list` | Returns all declared `resource` blocks with their URI, description, and mimeType |
| `resources/read` | Runs the matching resource body, returns the result as a resource content block |
| `prompts/list` | Returns all declared `prompt` blocks with their name, description, and arg schema |
| `prompts/get` | Deserializes args, runs the matching prompt body, returns the rendered message |

### 12.3 Session Isolation

Each MCP client session is isolated. State variables in tool bodies are local to their invocation — they are not shared across calls. If persistent state is needed across tool invocations, store it in the database using frame.data.

---

## 13. Host Bridge Functions

frame.mcp uses the `host:mcp` bridge namespace internally. Developers do not call these functions directly — they are invoked by the plugin runtime. They are documented here for implementers of new runtime hosts.

| Bridge function | Params | Returns | Description |
|----------------|--------|---------|-------------|
| `mcp.stdio_read()` | — | `string` | Read the next newline-delimited JSON-RPC message from stdin. Blocks until a complete message is available. Returns empty string on EOF. |
| `mcp.stdio_write(message)` | `string` | — | Write a newline-terminated JSON-RPC message to stdout. |
| `mcp.http_serve(port, host)` | `integer, string` | — | Bind and start the HTTP listener on the given port and host. Blocks until the server stops. |
| `mcp.http_accept()` | — | `string` | Wait for the next HTTP request and return it as a JSON string containing the method, headers, and body. |
| `mcp.sse_send(clientId, event)` | `string, string` | — | Send an SSE event to a connected HTTP client identified by `clientId`. |
| `mcp.log(level, message)` | `string, string` | — | Write a structured log entry. Always routes to stderr regardless of transport, so it never corrupts the stdio stream. `level` is `"debug"`, `"info"`, `"warn"`, or `"error"`. |

### 13.1 Log Routing

The `mcp.log()` bridge always writes to stderr, even when the stdio transport is active. This is a hard guarantee — the plugin runtime never writes log output to stdout. Developers who want to emit informational messages from within tool bodies can call `mcp.log()` directly:

```clean
tool "diagnose":
    description: "Run server diagnostics"
    params:
        verbose: boolean = false
    mcp.log("info", "Starting diagnostics")
    integer userCount = User.count()
    if params.verbose:
        mcp.log("debug", "User count: " + userCount.toString())
    return json({ userCount: userCount, status: "ok" })
```

---

## 14. Integration with Other Plugins

### 14.1 frame.data — ORM Queries in Tool Bodies

frame.mcp integrates naturally with frame.data. ORM queries, model lookups, inserts, and updates are available directly inside tool and resource bodies. The frame.data plugin must be declared in `main.cln` alongside frame.mcp.

```clean
// main.cln
target:
    platform: server
    plugins:
        frame.mcp
        frame.data
```

Model types defined in `app/data/models/` are available by name in tool params and variable declarations:

```clean
tool "list_users":
    description: "List active users"
    params:
        role: string = "any"
    list<User> users = User.find:
        where:
            active == true
    return json(users)
```

If frame.data is not declared in `main.cln`, ORM calls in tool bodies will produce a compile error.

### 14.2 frame.server — Coexistence

frame.mcp and frame.server can coexist in the same project. They serve different clients on different ports:

- frame.server handles HTTP traffic from browsers and API consumers on the main application port.
- frame.mcp (with `http` transport) listens on a separate port for AI agent connections.
- frame.mcp (with `stdio` transport) communicates over stdin/stdout as a subprocess — no port conflict.

There is no dependency between frame.server and frame.mcp. Declaring both in `main.cln` is valid.

### 14.3 frame.auth — Authentication Context

Standard frame.auth session and JWT authentication does not apply to MCP servers, because MCP clients are AI agents, not browsers with cookies or login flows. When authentication is needed, use the `apiKey:` field in the `mcp:` block (see Section 4.1). The API key is a pre-shared secret passed in the Authorization header.

If finer-grained access control is required (e.g., per-tool permissions per API key), implement it in the tool body by inspecting a database record associated with the key value retrieved from `env.get("MCP_API_KEY")`.

---

## 15. Complete Example

A realistic `app/mcp/blog-tools.cln` demonstrating all three primitives in a blog application with frame.data integration:

```clean
// app/mcp/blog-tools.cln
mcp "blog-tools":
    version: "1.0.0"
    description: "Tools for AI agents to read and manage blog content"
    transport: stdio

    // ─── TOOLS ────────────────────────────────────────────────

    tool "list_posts":
        description: "List blog posts with optional filtering by published status and pagination"
        params:
            published: boolean = true
            limit: integer = 20
            offset: integer = 0
        list<Post> posts = Post.find:
            where:
                published == params.published
            order:
                createdAt desc
            limit: params.limit
            offset: params.offset
        return json(posts)

    tool "get_post":
        description: "Get a single blog post by ID, with all its fields"
        params:
            id: integer
        Post? post = Post.first:
            where:
                id == params.id
        if post == null:
            return error("Post not found", code: "NOT_FOUND")
        return json(post)

    tool "create_post":
        description: "Create a new blog post in draft state (published = false)"
        params:
            title: string
            content: string
            authorId: integer
            tags: list<string> = []
        Post p = Post.insert:
            title = params.title
            content = params.content
            authorId = params.authorId
            published = false
            createdAt = time.now()
            updatedAt = time.now()
        return json(p)

    tool "publish_post":
        description: "Publish a draft post, making it publicly visible"
        params:
            id: integer
        Post? post = Post.first:
            where:
                id == params.id
        if post == null:
            return error("Post not found", code: "NOT_FOUND")
        if post.published:
            return error("Post is already published", code: "ALREADY_PUBLISHED")
        Post updated = Post.update:
            where:
                id == params.id
            published = true
            updatedAt = time.now()
        return json(updated)

    tool "delete_post":
        description: "Permanently delete a post and all its comments"
        params:
            id: integer
        Post? post = Post.first:
            where:
                id == params.id
        if post == null:
            return error("Post not found", code: "NOT_FOUND")
        Comment.delete:
            where:
                postId == params.id
        Post.delete:
            where:
                id == params.id
        return text("Post " + params.id.toString() + " and all its comments deleted.")

    tool "add_comment":
        description: "Add a comment to a blog post"
        params:
            postId: integer
            authorName: string
            body: string
        Post? post = Post.first:
            where:
                id == params.postId
        if post == null:
            return error("Post not found", code: "NOT_FOUND")
        Comment c = Comment.insert:
            postId = params.postId
            authorName = params.authorName
            body = params.body
            createdAt = time.now()
        return json(c)

    tool "list_comments":
        description: "List all comments on a specific post"
        params:
            postId: integer
        list<Comment> comments = Comment.find:
            where:
                postId == params.postId
            order:
                createdAt asc
        return json(comments)

    // ─── RESOURCES ────────────────────────────────────────────

    resource "posts://recent":
        description: "The 10 most recently published posts — useful for AI context"
        mimeType: "application/json"
        list<Post> posts = Post.find:
            where:
                published == true
            order:
                createdAt desc
            limit: 10
        return json(posts)

    resource "posts://drafts":
        description: "All unpublished draft posts awaiting review"
        mimeType: "application/json"
        list<Post> drafts = Post.find:
            where:
                published == false
            order:
                updatedAt desc
        return json(drafts)

    resource "stats://overview":
        description: "High-level blog statistics: post count, comment count, author count"
        mimeType: "application/json"
        integer totalPosts = Post.count()
        integer publishedPosts = Post.count:
            where:
                published == true
        integer totalComments = Comment.count()
        integer totalAuthors = User.count:
            where:
                role == "author"
        return json({
            totalPosts: totalPosts,
            publishedPosts: publishedPosts,
            draftPosts: totalPosts - publishedPosts,
            totalComments: totalComments,
            totalAuthors: totalAuthors
        })

    // ─── PROMPTS ──────────────────────────────────────────────

    prompt "summarize_post":
        description: "Prompt to generate a social media summary of a blog post"
        args:
            post_id: integer
            platform: string = "twitter"
            max_chars: integer = 280
        Post? post = Post.first:
            where:
                id == args.post_id
        if post == null:
            return error("Post not found")
        return "Summarize the following blog post for {{ platform }} in {{ max_chars }} characters or fewer. Be engaging and include a call to action.\n\nTitle: {{ post.title }}\n\nContent:\n{{ post.content }}"

    prompt "draft_post_outline":
        description: "Prompt to generate a structured outline for a new blog post"
        args:
            topic: string
            audience: string = "general readers"
            length: string = "medium"
        return "Create a detailed outline for a {{ length }}-length blog post about '{{ topic }}' targeted at {{ audience }}. Include an introduction, 3-5 main sections with subpoints, and a conclusion. Format as a numbered list."

    prompt "reply_to_comment":
        description: "Prompt to draft an author reply to a reader comment"
        args:
            comment_id: integer
            tone: string = "friendly and professional"
        Comment? comment = Comment.first:
            where:
                id == args.comment_id
        if comment == null:
            return error("Comment not found")
        return "Write a {{ tone }} author reply to the following reader comment. Keep it under 3 sentences and acknowledge the reader's point directly.\n\nComment by {{ comment.authorName }}:\n{{ comment.body }}"
```

---

## 16. Plugin.toml Contract

The `frame.mcp` plugin declares the following contract in its `plugin.toml`. This is consumed by the compiler to understand which block keyword to dispatch and which bridge namespace the plugin requires.

```toml
[plugin]
name = "frame.mcp"
version = "1.0.0"
description = "MCP server plugin — tools, resources, and prompts for AI agents"
min_compiler_version = "0.31.0"

[block]
block_handle = "mcp"
implicit_import = true
owned_paths = ["app/mcp/"]

[bridge]
namespace = "mcp"
functions = [
    { name = "mcp.stdio_read",  params = [],                            returns = "string"  },
    { name = "mcp.stdio_write", params = ["string"],                    returns = "void"    },
    { name = "mcp.http_serve",  params = ["integer", "string"],         returns = "void"    },
    { name = "mcp.http_accept", params = [],                            returns = "string"  },
    { name = "mcp.sse_send",    params = ["string", "string"],          returns = "void"    },
    { name = "mcp.log",         params = ["string", "string"],          returns = "void"    },
]

[capabilities]
tools = true
resources = true
prompts = true
```

**Field reference:**

| Field | Value | Meaning |
|-------|-------|---------|
| `block_handle` | `"mcp"` | The top-level keyword that triggers this plugin during compilation |
| `implicit_import` | `true` | Files in `owned_paths` are automatically compiled without per-file imports |
| `owned_paths` | `["app/mcp/"]` | The compiler routes all `.cln` files in this folder to this plugin |
| `bridge_namespace` | `"mcp"` | Host bridge functions are prefixed with `mcp.` |

---

**End of Document 15 — Frame MCP Specification (tools, resources, prompts, stdio and HTTP+SSE transports)**
