# Pending Tests

Test files that cannot be written correctly today because they depend on an
upstream fix. Every entry names the upstream error code and the intended
purpose of the test.

**Format**: one bullet per file. The leading path must match the file the
placeholder guard should skip. Nothing else in the line is parsed.

```
- <relative-path> — <ERROR_CODE>: <one-line reason>
```

**Rules**:
1. An entry belongs here only while an upstream fix is genuinely blocking.
   Once the fix ships, the entry is deleted in the same PR that writes the
   real test.
2. The file at the listed path must still emit `PASS:` on the paths it can
   exercise. `PENDING.md` is not a licence to check in nothing.
3. Adding an entry requires a real error code (see
   errors.cleanlanguage.dev). "TBD" is rejected in review.
4. This file must shrink toward empty. Every remediation PR that lands
   real tests deletes the corresponding entries here.

See [../system-documents/testing/TEST_STRATEGY.md §5](../system-documents/testing/TEST_STRATEGY.md) for policy.

## Current entries

None allowlisted. Every test file in the tree today calls a real API and
asserts on a real value.

## Upstream blocker: compiler implicit-activation bug (#73cfeba24d14)

Coverage-matrix tests for **frame.canvas** (7 files), **frame.mcp** (4 files),
and **frame.client** (7 files) are deferred pending upstream compiler fixes.
The empirical picture:

- All four "broken" plugins — auth, canvas, client, mcp — compile fine when
  a file does NOT declare `plugins:` explicitly (implicit activation kicks in,
  runs a different code path, no scaffolding bugs).
- The same files FAIL when `plugins:` is declared explicitly, because the
  compiler's typed-emission wrapping of `emit_module_helpers_typed` produces
  code referencing undeclared variables (`dt`, `__cv`, `__rfr`, `r`,
  `__res_users`) or unsupported binops.

The blocking bugs:

- **#73cfeba24d14** COMPILER-IMPLICIT-PLUGIN-ACTIVATION — the systemic
  issue. Files should require explicit `plugins:` declarations; the
  compiler currently activates plugins automatically when a block-name
  matches an installed plugin. This blinds the MCP server, IDE extension,
  and external plugin authors, and hides the module_helpers scaffolding
  bug below.
- **#43ee45647d28** FRAME-CANVAS-MODULE-HELPERS-BROKEN — canvas fails
  under explicit activation. Diagnostic and repro attached to the bug.
- **#7536dcc58643** FRAME-MCP-MODULE-HELPERS-BROKEN — same class as canvas.
- **#4c71acb99571** FRAME-CLIENT-MODULE-HELPERS-BROKEN — same class.
- **#3654d453ebda** FRAME-CLIENT-FORM-PARSER — separate parser bug in the
  client plugin's `form:` block. Independent of implicit activation.

**Once #73cfeba24d14 lands** (implicit activation removed), the framework's
tests currently in the tree will need `plugins:` declarations added to any
file that uses a plugin block. That is a mechanical grep-and-insert pass
and does not block progress; it is called out here so the change is not a
surprise.

**Important:** the pre-emptive fix (adding `plugins:` today) is NOT
straightforward — during v2.12.160 remediation, empirical testing showed
that adding `plugins:` to files using `frame.auth`, `frame.canvas`,
`frame.client`, `frame.mcp`, `frame.server`, or `frame.data` breaks the
compile due to related scaffolding bugs (the `module_helpers_typed`
emission bug is broader than just the 4 originally-reported plugins). Only
files using `frame.jobs`, `frame.locale`, or standalone `html:` blocks can
safely declare `plugins:` today. Tests in the tree that use those safe
combinations already have `plugins:` declarations; the rest await the
upstream fixes and will be updated in a mechanical pass at that time.

**Once the plugin-side module_helpers bugs are fixed** (#43ee45647d28,
#7536dcc58643, #4c71acb99571): write the coverage-matrix tests for canvas
(7), mcp (4), and client (7) per TEST_STRATEGY.md §6.

**Once #3654d453ebda is fixed**: write the client form: test.

## When you land a fix

1. Re-verify the minimal reproduction in the corresponding `report_error`
   entry compiles cleanly under the current state (with explicit `plugins:`).
2. Write the plugin's coverage-matrix tests per TEST_STRATEGY.md §6.
3. Run `python3 scripts/check-test-placeholders.py` to confirm the guard
   is still green after your additions.
4. Update the section above by moving the resolved entry out.
