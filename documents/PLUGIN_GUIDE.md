# Clean Plugin Usage: User‑Facing Definition

## 1. What Is a Clean Plugin? (Developer Mental Model)
A **Clean plugin** is the simplest way for a Clean Language developer to gain new capabilities in the language without learning advanced internals.

A plugin:
- Adds **new blocks** (DSL blocks) you can write inside `.cln` files.
- Expands those blocks into **normal Clean code** before compilation.
- Requires no knowledge of ASTs, WASM pipelines, or Rust.

**As a user:** you just install a plugin, import it, and start using its blocks.

Example:
```clean
configuration:
    project:
        name = "my-app"

    plugins:
    frame.server
    frame.data
```

This file lives in the **parent folder** of your Clean project (for example, above `src/`).

The compiler transforms `endpoints:` into regular Clean functions that register routes.

---

## 2. How Developers Use Plugins

### 2.1. Simple Workflow
1. **Install Clean + a plugin using `cleen`:**
```bash
cleen install latest
cleen plugin add frame.server
```

2. **Declare the plugin in your `main.cln` `plugins:` block:**
```clean
plugins:
    frame.server
```

Plugins with `implicit_import = true` in their `plugin.toml` allow files inside their owned folders (e.g. `app/server/api/` for `frame.server`) to skip individual import statements. The plugin declaration in `main.cln` is still required — what's implicit is the per-file import, not the plugin declaration itself.

3. **Use its new blocks:**
```clean
endpoints:
    GET "/users":
        return json(User.find:)
```

4. **Compile as usual:**
```bash
cln build
```

The plugin converts the DSL into normal Clean code, and compilation continues normally.

### 2.2. Types of Plugins
- **Web/API (`frame.server`):** `server`, `endpoints`
- **Data/ORM (`frame.data`):** `data` (keyword-based model definition)
- **UI/Pages (`frame.ui`):** `component`, `screen`, `page`, `html`
- **Auth (`frame.auth`):** `auth`, `protected`, `login`, `roles`
- **Canvas (`frame.canvas`):** `canvasScene`, `draw`, `onFrame`

Each plugin simply adds readable blocks that make Clean more expressive.

---

## 3. Plugin Commands (Using `cleen`)
Plugins are managed through the version manager **cleen**, the same tool used to install Clean Language.

### 3.1. Existing Commands
```bash
cleen install latest
cleen use 1.2.3
cleen list
```

### 3.2. New Plugin Commands
#### Install a plugin
```bash
cleen plugin add frame.server
```

#### Remove a plugin
```bash
cleen plugin remove frame.server
```

#### List installed plugins
```bash
cleen plugin list
```

#### Search for plugins
```bash
cleen plugin search httpserver
```

### 3.3. Project Plugin Configuration
A project may include a **Clean configuration file** named `configuration.cln` written in Clean-style syntax:
```clean
project:
    name = "my-app"

plugins:
    frame.server
    frame.data
```

Then you can run:
```bash
cleen plugin sync
```

This installs everything needed for the project.

---

## 4. How Plugins Work Internally (Simplified Explanation)
1. The compiler parses your `.cln` file.
2. When it encounters a DSL block like `endpoints:`, it hands that node to the plugin that registered that block.
3. The plugin expands the block into normal Clean functions.
4. The compiler continues normally (types, CIR, WASM).
5. The WASM module uses the Clean Frame host runtime for HTTP, DB, UI, etc.

For developers: *“I write a friendly DSL block; the plugin turns it into real code.”*

---

## 5. Clean Manager (`cleen`) as the Distribution Hub

### 5.1. Current Behavior
`cleen` installs compiler versions under:
```
~/.cleen/versions/<version>/
```
And exposes the active compiler via `~/.cleen/bin/cln`.

### 5.2. Extended Behavior for Plugins and Framework
`cleen` will now distribute:
- The **Clean compiler**.
- **Plugins**.
- The **Clean Framework (Frame)**.

### 5.3. Installing Framework Bundles
```bash
cleen install clean@1.2.3      # Pure compiler
cleen install frame@1.2.3      # Compiler + official plugins + runtime
```

### 5.4. Framework Commands
```bash
cleen framework new my-app      # Create a new project
cleen framework upgrade         # Update framework + plugins
```

---

## 6. Why This Design Is Friendly
- One tool (`cleen`) installs everything.
- Plugins feel like “new blocks you can write”.
- Simple vocabulary: `add`, `remove`, `sync`, `install`, `use`.
- Low learning curve.
- Easy onboarding: clone project → `cleen plugin sync` → code.

---

This is the high‑level user‑facing definition of the plugin system and its interaction with cleen. More detailed technical architecture, lifecycle hooks, plugin packaging, and host integration can be added as next steps.

---

## 7. Framework Project Structure and Location Awareness

The **Clean Framework** (Frame) assumes a simple, opinionated project structure. When you create a framework project, this structure is created automatically and is **location aware**, so tools always know where to look.

### 7.1. Default Framework Project Layout

When you run:

```bash
cleen framework new my-app
```

you get something like (actual folders depend on the plugins and preset you choose):

```text
my-app/
  main.cln
  app/
    server/
      api/
        users.cln
        orders.cln
      services/
      middleware/
    pages/              # used by frame.ui (SSR pages)
      home.html
      about.html
    components/         # used by frame.ui (reusable components)
      header.cln
      footer.cln
    data/               # used by frame.data (ORM models, queries, migrations)
      models/
        User.cln
      queries/
      migrations/
      repositories/
    auth/               # used by frame.auth (auth configuration)
    canvas/             # used by frame.canvas (canvas scenes, sprites, audio)
      scenes/
      sprites/
      audio/
```

**Key points:**
- `main.cln` lives at the **root** and configures the project, compiler, plugins and framework.
- The `plugins:` block in `main.cln` declares which plugins are active.
- `app/server/` and its subfolders are owned by `frame.server` (endpoints, services, middleware).
- `app/data/` and its subfolders are owned by `frame.data`.
- `app/auth/` is owned by `frame.auth`.
- `app/web/pages/`, `app/web/components/`, and `app/web/layouts/` are owned by `frame.ui`.
- `app/canvas/` and its subfolders are owned by `frame.canvas`.
- All of these are **plugin-owned folders** — once a plugin is declared in `main.cln`, files placed in its owned folder are processed by that plugin without needing individual import statements.

### 7.2. Root `main.cln` and `start`

At the root, the framework expects an initial file, typically `main.cln`, which defines the **`start`** block used as the application entrypoint.

```clean
start:
    // Application startup logic
    // e.g. register routes, initialize services, etc.
```

The framework runtime calls `start` after loading the WASM module.

This block follows Clean style: a named block followed by `:` and an indented body.

### 7.3. Plugin-defined folders

Each plugin can define the folder(s) it uses. When a framework project is created, the CLI inspects the selected preset and the plugins declared in `configuration.cln` and **creates only the folders needed by those plugins**. The Framework + compiler do the rest, so the user **does not need boilerplate** inside those files.

Examples:

- `frame.server` (web/API plugin) owns:
  - `app/server/` → HTTP endpoints, services, and middleware.
  - `app/server/api/` → endpoint handler files.
  - `app/logic/` → service layer files.
  - `app/server/middleware/` → middleware files.
- `frame.ui` (UI/HTML plugin) owns:
  - `app/web/pages/` → SSR page templates.
  - `app/web/components/` → reusable UI components.
  - `app/web/layouts/` → shared layout wrappers.
- `frame.data` (ORM plugin) owns:
  - `app/data/` → data models, queries, migrations, and repositories.
  - `app/data/models/` → model definitions using the `data` keyword.
  - `app/data/` → reusable query blocks.
  - `app/data/migrations/` → schema migration files.
  - `app/data/` → repository pattern files.
- `frame.auth` (auth plugin) owns:
  - `app/auth/` → authentication and authorization configuration.
- `frame.canvas` (canvas plugin) owns:
  - `app/canvas/` → canvas applications and scenes.
  - `app/canvas/scenes/` → scene definitions using `canvasScene`.
  - `app/canvas/sprites/` → sprite and asset definitions.
  - `app/canvas/audio/` → audio assets and configuration.

**Behavior:**
- The plugin declares: *“I own folder `pages/` and file patterns `*.html`.”*
- The framework automatically:
  - Finds those files.
  - Routes them to the plugin for processing.
  - Includes the results in the build/runtime, with no extra code in `main.cln`.

### 7.4. No per-file boilerplate

Because plugins are folder-aware, once a plugin is declared in `main.cln`, a developer can create files in the owned folder and they will be picked up automatically:

```text
app/web/pages/
  home.html
  contact.html
```

No need to manually register these pages. For example, `frame.ui` treats each `*.html` in its owned folder as a route or named template, depending on its rules.

This means:

- **No per-file import statements** like `import frame.ui` in every `.cln` file.
- **No boilerplate registration** like "registerPage(home)".
- Just declare the plugin in `main.cln`, create the file in the right folder, and the framework handles the rest.

### 7.5. Example: `.html` processing

For UI/HTML-oriented plugins, we define a special extension convention:

- Files ending in `.html` contain **HTML with custom tags**.
- These tags are processed **before the file is retrieved/served**.

Example file: `app/web/pages/home.html`

```html
<html>
  <body>
    <AppHeader />

    <h1>Welcome, {{ user.name }}</h1>

    <If loggedIn>
      <p>Your last login was {{ user.lastLogin }}</p>
    </If>

    <AppFooter />
  </body>
</html>
```

The plugin pipeline might:

1. Parse the `.html` file.
2. Process custom tags such as `<AppHeader />`, `<If ...>`, or `{{ user.name }}`.
3. Convert this into Clean or template code that can be compiled and rendered at runtime.

From the developer perspective:

- They just write HTML with some extra tags.
- The plugin automatically wires it into the framework.

### 7.6. Minimal Example Project

Below is a full minimal example of a framework project using the conventions above.

**`main.cln` (at project root)**

```clean
configuration:
    project:
        name = "my-app"

    compiler:
        version = "latest"

    plugins:
        frame.server
        frame.ui

    framework:
        enabled = true
        name = "frame"
        preset = "web-api"
        runtime = "node"
```

**`main.cln` (entrypoint at project root)**

```clean
start:
    // Startup logic
    // The framework discovers backend and page files via plugin folder ownership.
    // Plugins declared in main.cln process all files in their owned folders.
    // No per-file import statements needed.
```

**`app/server/api/users.cln` (API endpoints owned by `frame.server`)**

```clean
endpoints:
    GET "/users":
        list<User> users = User.find:
        return json(users)
```

**`app/web/pages/home.html` (HTML + custom tags owned by `frame.ui`)**

```html
<html>
  <body>
    <AppHeader />
    <h1>Welcome to My App</h1>

    <If loggedIn>
      <p>Hello, {{ user.name }}!</p>
    </If>

    <AppFooter />
  </body>
</html>
```

In this example:

- `main.cln` declares the project, compiler, plugins and framework via the `plugins:` block.
- `main.cln` defines `start`, the entrypoint block executed by the framework runtime.
- `app/server/api/users.cln` is discovered by `frame.server` (its owned folder) and turned into real endpoint registration code.
- `app/web/pages/home.html` is discovered by `frame.ui` (its owned folder) and compiled into a renderable page template.

Developers do not need extra boilerplate; they only need to respect the folder structure and file conventions.


