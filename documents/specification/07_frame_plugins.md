# Frame Plugins Specification (07)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.0  
**Location:** `/docs/specification/07_frame_plugins.md`

---

## 1. Introduction

**Frame Plugins** allow developers to extend the framework without changing its core. A plugin can add UI components (custom tags), new CLI commands, server routes, build steps, or Host Bridge adapters.

**Design goals:**
- **Simple to author** (Clean-first), **safe to run** (WASM sandbox), and **easy to compose** (declarative hooks).
- **Deterministic contracts** so both humans and AI tools can reason about behavior.

---

## 2. Plugin Anatomy

A plugin is a directory with a manifest plus optional modules:
```
plugins/
  charts/
    plugin.cln           # manifest + hook registrations
    ui/LineChart.cln     # UI component(s)
    server/routes.cln    # extra API routes
    cli/add_chart.cln    # CLI command(s)
    data/hooks.cln       # ORM/data hooks
    package.json         # (optional) npm deps for host adapters
```

**plugin.cln (minimal):**
```clean
plugin Charts
    meta:
        name = "charts"
        version = "1.0.0"
        description = "Basic chart components"

    hooks:
        ui: registerTags
        cli: registerCLI
```

---

## 3. Lifecycle & Hook Phases

The framework detects plugins at build/serve time and calls hooks in a fixed order:

1) **discover** → scan `/plugins/*/plugin.cln`
2) **init** → load plugin metadata and options
3) **cli** → register CLI commands
4) **ui** → register custom tags/components
5) **server** → register routes/middleware
6) **data** → register ORM hooks
7) **build** → allow static asset emission or transforms

**Execution guarantees:**
- Hooks run in an isolated WASM context with read-only access to project metadata unless write permission is granted explicitly (see Security).
- Order between plugins is **by name** (lexicographic) unless `requires` specifies dependencies.

---

## 4. UI Hooks

Register custom tags and link them to Clean components.

```clean
functions:
    registerTags(tags)
        tags.register("charts-line", "plugins/charts/ui/LineChart.cln")
        tags.register("charts-bar",  "plugins/charts/ui/BarChart.cln")
```

**Component example (LineChart.cln):**
```clean
component LineChart
    props:
        data: list<integer>
    functions:
        Widget render()
            return (
                <canvas class="chart" data-points="{json(data)}"></canvas>
            )
```

> Client hydration can be enabled via `client="on"` when used, or `hydrate:` in the component.

---

## 5. CLI Hooks

Add developer commands with a simple interface.

```clean
functions:
    registerCLI(cli)
        cli.command("charts:add")
            .describe("Insert a chart component into a target page")
            .option("--type", default: "line")
            .option("--page")
            .action(addChart)

    void addChart(map<string, any> args)
        # insert tag in /app/pages/<page>.cln based on args
```

CLI commands should print machine-readable output when `--json` is present:
```json
{ "ok": true, "msg": "charts:add completed", "page": "dashboard" }
```

---

## 6. Server Hooks

Expose additional API routes or middleware.

```clean
functions:
    registerServer(server)
        server.route("GET", "/api/charts/ping", ping)

    Response ping(Request r)
        return json({ ok: true, time: now() })
```

Middleware can be used for auth, rate limiting, or logging.

---

## 7. Data Hooks

React to ORM lifecycle events (create/update/delete) or transform queries.

```clean
functions:
    registerData(data)
        data.on("user.created", onUserCreated)
        data.on("post.deleted", onPostDeleted)

    void onUserCreated(User u)
        bridge.log.info({ event: "user.created", id: u.id })

    void onPostDeleted(Post p)
        bridge.log.warn({ event: "post.deleted", id: p.id })
```

> Hooks run **after** the transaction commits unless `data.on(..., { timing: "before" })` is specified.

---

## 8. Build Hooks & Assets

Emit static assets or transform outputs.

```clean
functions:
    registerBuild(build)
        build.emit("public/charts.css", ":root{ --chart-gap: 8px; }")
        build.copy("plugins/charts/public/", "public/plugins/charts/")
```

You can also add entries to the **islands manifest** for client hydration.

---

## 9. Plugin Manifest Schema

**plugin.cln** may declare metadata and dependencies:
```clean
plugin Charts
    meta:
        name = "charts"
        version = "1.0.0"
        description = "Basic chart components"
        author = "Your Name"
        license = "Apache-2.0"
    requires:
        - "core>=1.0.0"
        - "ui>=1.0.0"
```

**Internal normalized JSON (AI-readable):**
```json
{
  "name": "charts",
  "version": "1.0.0",
  "description": "Basic chart components",
  "hooks": ["ui", "cli"],
  "requires": ["core>=1.0.0", "ui>=1.0.0"]
}
```

---

## 10. Security Model

- Plugins run in **WASM sandboxes**.
- Default permissions are **read-only**. To write files or access network/DB, a plugin must declare permissions:

```clean
plugin Charts
    permissions:
        fs.write: ["public/plugins/charts/*"]
        net.http: ["https://cdn.example.com/*"]
```

- The Host Bridge enforces allowlists (`fs`, `http`, `env`).
- CLI prompts (or config) must approve elevated permissions during install.

---

## 11. Versioning & Compatibility

- Plugins follow **semver** (`MAJOR.MINOR.PATCH`).
- Declare required **Frame core version** in `requires`.
- The CLI refuses incompatible plugins and prints a clear error:
```json
{ "ok": false, "err": { "code": "PLUGIN_INCOMPATIBLE", "message": "requires core>=1.2.0" } }
```

---

## 12. Testing Plugins

- **Unit:** test hook functions in isolation with a mock host.
- **Integration:** run `frame serve` with the plugin enabled and verify routes/UI.
- **Snapshots:** compare SSR HTML or generated files.

Test location suggestion: `/tests/plugins/<name>/`.

---

## 13. Publishing & Distribution

- Package your plugin directory and publish to your preferred registry (git tag or archive).
- Provide a simple installer script or `frame plugin:add <repo>` command.
- Keep `plugin.cln` small and deterministic for AI agents to parse.

---

## 14. AI Development Notes

This file exposes **deterministic hook shapes** and a **normalized manifest** so AI tools can:
- Discover available hooks and capabilities.
- Generate safe boilerplate (UI tags, CLI commands, routes).
- Validate permission declarations and detect unsafe patterns.

When prompting an AI agent, include `07_frame_plugins.md` + `03_frame_server.md` (for routes) + `05_frame_ui.md` (for tags) as context.

---

## 15. File Locations

- Plugin root: `/plugins/<plugin-name>/`
- Manifest: `/plugins/<plugin-name>/plugin.cln`
- UI components: `/plugins/<plugin-name>/ui/*.cln`
- Server routes: `/plugins/<plugin-name>/server/*.cln`
- CLI commands: `/plugins/<plugin-name>/cli/*.cln`
- Data hooks: `/plugins/<plugin-name>/data/*.cln`
- Public assets: `/plugins/<plugin-name>/public/*`

---

**End of Document 07**

