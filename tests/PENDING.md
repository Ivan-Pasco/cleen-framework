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

None. Every test file in the tree today calls a real API and asserts on a
real value. Placeholder tests for frame.canvas (bug ID `#32f60f7eb690` —
canvasScene: emits undefined `dt`/`__cv`/`__rfr` helpers), frame.client
(bug ID `#cd7d8a080e66` — `form:`/`load:` blocks emit broken scaffolding),
and frame.mcp (bug ID `#c9b307819453` — `mcp "name":` emits ~136 undefined
`r` errors) have been deleted rather than left as placeholders. The
per-plugin coverage matrix in TEST_STRATEGY.md §6 tracks the missing files;
they will be written in the same PR that closes each upstream bug.

If you land a fix for any of those three bugs, please:

1. Re-verify the minimal reproduction in the corresponding `report_error`
   entry compiles cleanly.
2. Write the plugin's coverage-matrix tests per TEST_STRATEGY.md §6.
3. Run `python3 scripts/check-test-placeholders.py` to confirm the guard
   is still green after your additions.
4. Delete this `## Current entries` section's descriptive note if all
   three bugs are closed.
