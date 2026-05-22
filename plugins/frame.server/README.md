# frame.server Plugin

HTTP server plugin for Clean Language. Provides DSL blocks for building HTTP servers with routing, request handling, and authentication.

**Requires:** clean-server runtime

## Blocks

### endpoints

Defines HTTP endpoint handlers using a declarative syntax.

**Example:**
```clean
import:
    frame.server

endpoints:
    GET "/" -> homePage()
    GET "/api/users" -> listUsers()
    GET "/api/users/:id" -> getUser()
    POST "/api/users" -> createUser()
```

### server

Creates an HTTP server configuration.

**Attributes:**
- `port` (optional, default: 8080) - Port to listen on

**Example:**
```clean
import:
    frame.server

server: port=3000
    route: method="GET" path="/"
        return {"message": "Hello World"}
```

### route

Defines an HTTP route handler.

**Attributes:**
- `method` (required) - HTTP method (GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD)
- `path` (required) - URL path pattern (must start with /)

**Example:**
```clean
route: method="GET" path="/users/:id"
    id = _req_param("id")
    return getUserById(id)
```

### middleware

Adds middleware to the request chain.

## Bridge Functions

Server-specific functions provided by clean-server:

### HTTP Server
- `_http_listen(port)` - Start the HTTP server
- `_http_route(method, path, handler_idx)` - Register a route handler
- `_http_route_protected(method, path, handler_idx, role)` - Register protected route

### Request Context
- `_req_param(name)` - Get path parameter
- `_req_query(name)` - Get query parameter
- `_req_header(name)` - Get request header
- `_req_body()` - Get request body
- `_req_method()` - Get HTTP method
- `_req_path()` - Get request path

### Authentication
- `_auth_get_session()` - Get current session as JSON
- `_auth_require_auth()` - Check if authenticated
- `_auth_require_role(role)` - Check if user has role
- `_auth_can(permission)` - Check permission
- `_auth_has_any_role(roles)` - Check if user has any role

## Building

```bash
./build.sh
```

## Installation

```bash
cp -r . ~/.cleen/plugins/frame.server/1.0.0/
```
