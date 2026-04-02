# Article Blog Example - Database Error Handling Fix

**Date:** 2025-12-25
**Issue:** Handler crashes when database is not configured
**Resolution:** Added proper error handling for database query responses

## Problem

The article-blog example (`examples/article-blog/app-db.cln`) crashed when accessing the homepage without a configured database:

```
Handler __route_handler_0 failed: error while executing at wasm backtrace:
    0: 0x31d1 - <unknown>!<wasm function 304>
```

**Root Cause:**
The code assumed `_db_query()` would return `{"rows": [...]}` structure, but when the database is not configured, it returns:
```json
{"ok": false, "err": {"code": "CONNECTION_ERROR", "message": "..."}}
```

The code tried to access `data.rows` directly, which was null/invalid, causing `list.get()` to crash.

## Solution

Added proper error response handling in both database query functions:

### Changes to `getArticleCardsFromDB()`:

```clean
// Parse JSON response
any data = json.tryTextToData(result)
string c = ""

if data == null
	c = c + "<div class=\"db-status error\"><p>Database query failed: Invalid response</p></div>"
	return c

// Check for database error response
any err = data.err
if err != null
	c = c + "<div class=\"db-status error\"><p>Database connection not configured. Please set DATABASE_URL environment variable.</p></div>"
	return c

// Access rows from successful response
any dataObj = data.data
if dataObj == null
	c = c + "<div class=\"db-status error\"><p>Database response missing data</p></div>"
	return c

any rows = dataObj.rows
if rows == null
	c = c + "<div class=\"db-status\"><p>No articles in database yet.</p></div>"
	return c
```

### Changes to `getArticleFromDB()`:

```clean
// Parse JSON response
any data = json.tryTextToData(result)
if data == null
	return getArticle404()

// Check for database error response
any err = data.err
if err != null
	return getArticle404()

// Access rows from successful response
any dataObj = data.data
if dataObj == null
	return getArticle404()

any rows = dataObj.rows
if rows == null
	return getArticle404()
```

### CSS for Error Display:

```css
.db-status.error {
	background: #fee;
	color: #a00;
}
```

## Database Response Structures

### Error Response (when database not configured):
```json
{
  "ok": false,
  "err": {
    "code": "CONNECTION_ERROR",
    "message": "Failed to get database connection: Database not configured..."
  }
}
```

### Success Response:
```json
{
  "ok": true,
  "data": {
    "rows": [
      {"slug": "...", "title": "...", ...},
      ...
    ],
    "affected_rows": 0
  }
}
```

## Testing

### Without Database (DATABASE_URL not set):

```bash
~/.cleen/bin/cln compile examples/article-blog/app-db.cln -o app-db.wasm --plugins
~/.cleen/server/0.2.2/clean-server app-db.wasm --port 3000
```

**Results:**
- ✅ Homepage: Shows error message "Database connection not configured..."
- ✅ /api/articles: Returns JSON error response
- ✅ /articles/:slug: Shows 404 page
- ✅ No crashes or WASM execution errors

### With Database (when DATABASE_URL is set):

The app will query the database and display articles from the `articles` table.

## Files Modified

- `examples/article-blog/app-db.cln` - Added error handling to both query functions
- `examples/article-blog/app-db.wasm` - Recompiled with fixes

## Related Components

- ✅ Compiler v0.20.8 - Working correctly
- ✅ clean-server v0.2.2 - Working correctly
- ✅ frame.data plugin v1.0.0 - Working correctly
- ✅ frame.server plugin v1.0.0 - Working correctly

## Lessons Learned

1. **Always check for error responses** - Bridge functions return standard envelope format `{"ok": bool, ...}`
2. **Null checks are not enough** - Need to check for `err` field to detect errors
3. **Nested data structure** - Successful responses have `data.data.rows`, not `data.rows`
4. **Graceful degradation** - Show helpful error messages instead of crashing

---

**Status:** ✅ RESOLVED
**Framework Version:** All components working correctly (compiler v0.20.8, server v0.2.2)
