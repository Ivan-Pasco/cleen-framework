# Update Frame Plugins After Compiler Fix

## Prerequisites

The compiler's plugin runtime must have the math functions added (see PROMPT_COMPILER_PLUGIN_MATH_FUNCTIONS.md).

Test that the compiler fix works:
```bash
# Rebuild compiler
cd /Users/earcandy/Documents/Dev/Clean\ Language/clean-language-compiler
cargo build --release

# Install updated compiler
~/.cleen/bin/cln --version  # Should show new version or rebuild date
```

## Task 1: Update frame.web Plugin with Dynamic Endpoint Parsing

The current frame.web plugin has hardcoded routes. Update it to dynamically parse the `endpoints:` block.

**File:** `plugins/frame.web/src/main.cln`

**Replace the current `expand_endpoints` function with:**

```clean
string expand_endpoints(string body)
	string handlers = "functions:" + nl()
	string routes = ""
	integer idx = 0
	string text = body

	// Find GET routes
	integer pos = text.indexOf("GET /")
	while pos != -1
		string path = extract_path(text, pos, "GET")
		string handler_body = extract_handler(text, pos)
		handlers = handlers + make_handler(idx, handler_body)
		routes = routes + make_route("GET", path, idx)
		idx = idx + 1
		text = text.substring(pos + 5, text.length())
		pos = text.indexOf("GET /")

	// Reset for POST routes
	text = body
	pos = text.indexOf("POST /")
	while pos != -1
		string path = extract_path(text, pos, "POST")
		string handler_body = extract_handler(text, pos)
		handlers = handlers + make_handler(idx, handler_body)
		routes = routes + make_route("POST", path, idx)
		idx = idx + 1
		text = text.substring(pos + 6, text.length())
		pos = text.indexOf("POST /")

	// Similar for PUT, DELETE, PATCH...
	// (implement same pattern)

	string startup = "start()" + nl()
	startup = startup + tab() + "integer status = 0" + nl()
	startup = startup + tab() + "printl(\"Registering routes...\")" + nl()
	startup = startup + routes
	startup = startup + tab() + "printl(\"Routes registered\")" + nl()

	return handlers + startup

string extract_path(string text, integer pos, string method)
	integer method_len = method.length() + 1
	string after = text.substring(pos + method_len, text.length())
	integer colon = after.indexOf(":")
	if colon == -1
		return "/"
	return after.substring(0, colon).trim()

string extract_handler(string text, integer pos)
	// Find the colon after the path
	integer colon = text.indexOf(":", pos)
	if colon == -1
		return tab() + tab() + "return \"\"" + nl()

	// Find the next route definition or end
	string after_colon = text.substring(colon + 1, text.length())
	integer next_get = after_colon.indexOf("GET /")
	integer next_post = after_colon.indexOf("POST /")
	integer next_put = after_colon.indexOf("PUT /")
	integer next_delete = after_colon.indexOf("DELETE /")

	integer end_pos = after_colon.length()
	if next_get != -1 and next_get < end_pos
		end_pos = next_get
	if next_post != -1 and next_post < end_pos
		end_pos = next_post
	if next_put != -1 and next_put < end_pos
		end_pos = next_put
	if next_delete != -1 and next_delete < end_pos
		end_pos = next_delete

	string handler_code = after_colon.substring(0, end_pos).trim()
	if handler_code.length() == 0
		return tab() + tab() + "return \"\"" + nl()

	return tab() + tab() + handler_code + nl()

string make_handler(integer idx, string body)
	string h = tab() + "string __route_handler_" + idx.toString() + "()" + nl()
	h = h + body + nl()
	return h

string make_route(string method, string path, integer idx)
	return tab() + "status = _http_route(\"" + method + "\", \"" + path + "\", " + idx.toString() + ")" + nl()
```

## Task 2: Rebuild and Install Plugin

```bash
cd /Users/earcandy/Documents/Dev/Clean\ Language/clean-framework/plugins/frame.web
~/.cleen/bin/cln compile src/main.cln -o plugin.wasm
cp plugin.wasm ~/.cleen/plugins/frame.web/1.0.0/
```

## Task 3: Update Examples to Use endpoints: Block

After the plugin is working, update examples to use the cleaner DSL syntax:

**Example: `examples/todo-app/main.cln`**

```clean
import frame.web

endpoints:
	GET /:
		return "Welcome to Todo App"

	GET /api/todos:
		return "[]"

	POST /api/todos:
		return "created"

	GET /api/todos/:id:
		return "Todo details"

	PUT /api/todos/:id:
		return "updated"

	DELETE /api/todos/:id:
		return "deleted"

	GET /health:
		return "ok"
```

## Task 4: Test All Examples

```bash
# Compile each example with plugins
for dir in endpoints-test todo-app auth-guards; do
    cd /Users/earcandy/Documents/Dev/Clean\ Language/clean-framework/examples/$dir
    ~/.cleen/bin/cln compile main.cln -o /tmp/${dir}.wasm --plugins
done

# Test running one
~/.cleen/server/1.2.0/clean-server /tmp/todo-app.wasm --port 3000 &
sleep 2
curl http://localhost:3000/api/todos
pkill -f clean-server
```

## Task 5: Update Documentation

Update `documents/specification/07_frame_plugins.md` to document:

1. The `endpoints:` block DSL syntax
2. Supported HTTP methods (GET, POST, PUT, DELETE, PATCH)
3. Route parameters (`:id` syntax)
4. Handler body format
5. Examples

## Success Criteria

- [ ] frame.web plugin compiles without errors
- [ ] Plugin dynamically parses endpoints from source code
- [ ] All examples using `endpoints:` block compile and run
- [ ] Routes are correctly registered with clean-server
- [ ] Documentation is updated
