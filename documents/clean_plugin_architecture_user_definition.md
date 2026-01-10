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
    frame.web
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
cleen plugin add frame.web
```

2. **Import the plugin in your `.cln` file:**
```clean
import:
    frame.web
```

3. **Use its new blocks:**
```clean
endpoints:
    GET "/users" -> listUsers
```

4. **Compile as usual:**
```bash
cln build
```

The plugin converts the DSL into normal Clean code, and compilation continues normally.

### 2.2. Types of Plugins
- **Web/API:** `endpoints:`, `routes:`
- **Data/ORM:** `data:`, `model:`
- **UI/Pages:** `component:`, `view:`
- **Utilities:** `config:`, `log:`, `jobs:`

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
cleen plugin add frame.web
```

#### Remove a plugin
```bash
cleen plugin remove frame.web
```

#### List installed plugins
```bash
cleen plugin list
```

#### Search for plugins
```bash
cleen plugin search web
```

### 3.3. Project Plugin Configuration
A project may include a **Clean configuration file** named `configuration.cln` written in Clean-style syntax:
```clean
project:
    name = "my-app"

plugins:
    frame.web
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
  configuration.cln
  main.cln
  api/
    users.cln
    orders.cln
  pages/              # used by UI/HTML plugins (example)
    home.html.cln
    about.html.cln
  components/         # used by UI component plugins (example)
    header.cln
    footer.cln
  data/               # used by data/ORM plugins (example)
    models.cln
```

**Key points:**
- `configuration.cln` lives at the **root** and configures the project, compiler, plugins and framework.
- `main.cln` is the **entry file**, where `start` is defined.
- `api/` is the default folder for API-related code (e.g. endpoints, handlers).
- Other folders (`pages/`, `components/`, `data/`) are examples of **plugin-owned folders**.

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

- `frame.web` (web/API plugin) might define:
  - `api/` → files where HTTP endpoints live.
- `frame.ui` (UI/HTML plugin) might define:
  - `pages/` → page templates.
  - `components/` → reusable UI components.
- `frame.data` (ORM plugin) might define:
  - `data/` → data models and relations.

**Behavior:**
- The plugin declares: *“I own folder `pages/` and file patterns `*.html.cln`.”*
- The framework automatically:
  - Finds those files.
  - Routes them to the plugin for processing.
  - Includes the results in the build/runtime, with no extra code in `main.cln`.

### 7.4. Automatic discovery: no boilerplate

Because plugins are location aware, a developer can simply create files in the right place and they will be picked up:

```text
pages/
  home.html.cln
  contact.html.cln
```

No need to manually register these pages. For example, `frame.ui` can treat each `*.html.cln` as a route or a named template, depending on its rules.

This means:

- **No boilerplate imports** like "registerPage(home)".
- Just create the file and let the plugin + framework do the registration.

### 7.5. Example: `.html.cln` processing

For UI/HTML-oriented plugins, we define a special extension convention:

- Files ending in `.html.cln` contain **HTML with custom tags**.
- These tags are processed **before the file is retrieved/served**.

Example file: `pages/home.html.cln`

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

1. Parse the `.html.cln` file.
2. Process custom tags such as `<AppHeader />`, `<If ...>`, or `{{ user.name }}`.
3. Convert this into Clean or template code that can be compiled and rendered at runtime.

From the developer perspective:

- They just write HTML with some extra tags.
- The plugin automatically wires it into the framework.

### 7.6. Minimal Example Project

Below is a full minimal example of a framework project using the conventions above.

**`configuration.cln` (at project root)**

```clean
configuration:
    project:
        name = "my-app"

    compiler:
        version = "latest"

    plugins:
        frame.web
        frame.ui

    framework:
        enabled = true
        name = "frame"
        preset = "web-api"
        runtime = "node"
```

**`main.cln` (entrypoint at project root)**

```clean
import:
    frame.web
    frame.ui

start:
    // Startup logic
    // The framework will discover API and page files automatically.
```

**`api/users.cln` (API endpoints owned by `frame.web`)**

```clean
endpoints:
    GET "/users" -> listUsers

listUsers:
    // Implementation of the handler
    // This block will be expanded by the web plugin into normal Clean code.
```

**`pages/home.html.cln` (HTML + custom tags owned by `frame.ui`)**

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

- `configuration.cln` declares the project, compiler, plugins and framework.
- `main.cln` defines `start`, the entrypoint block executed by the framework runtime.
- `api/users.cln` is discovered by `frame.web` and turned into real endpoint registration code.
- `pages/home.html.cln` is discovered by `frame.ui` and compiled into a renderable page template.

Developers do not need extra boilerplate; they only need to respect the folder structure and file conventions.


