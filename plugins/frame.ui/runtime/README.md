# Frame UI Runtime

This directory contains the browser runtime for Frame UI applications.

## Files

- `shell.html` - HTML template that hosts the application
- `loader.js` - WASM loader and event handler (framework code)

## How it works

1. **Compile** your Clean Language screens to WASM:
   ```bash
   cln compile app/screens/Main.cln -o dist/frontend.wasm --plugins
   ```

2. **Copy runtime** to your dist folder:
   ```bash
   cp runtime/shell.html dist/index.html
   cp runtime/loader.js dist/loader.js
   ```

3. **Customize** the shell.html placeholders:
   - `{{APP_TITLE}}` - Page title
   - `{{APP_STYLES}}` - CSS styles
   - `{{SCREEN_NAME}}` - Initial screen name
   - `{{WASM_FILE}}` - Path to frontend.wasm

4. **Serve** the dist folder with any static server

## For Fullstack Apps

Fullstack apps produce two WASM files:

```
dist/
├── index.html      (shell + loader)
├── loader.js       (runtime)
├── frontend.wasm   (browser - UI screens)
└── backend.wasm    (server - API endpoints)
```

Run the backend with clean-server:
```bash
clean-server backend.wasm -p 3000
```

Serve the frontend with any static server or configure clean-server to serve static files.
