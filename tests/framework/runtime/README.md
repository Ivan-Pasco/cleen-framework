# Runtime Tests

Real-bridge tests that exercise actual plugin behaviour via `clean-server`. These are the tests CI gates on.

The legacy folders (`unit/`, `integration/`, `e2e/`) contain ~110 placeholder tests that assert against string literals rather than calling real code (see [tests/CONVENTIONS.md](../../CONVENTIONS.md) §9). They are being converted phase by phase; until conversion is complete, CI only runs the `runtime/` subtree.

## Layout

```
runtime/
├── plugins/
│   ├── auth/          # frame.auth: token sign/verify, sessions, roles
│   ├── data/          # frame.data: real query execution
│   ├── server/        # frame.server: HTTP route registration + dispatch
│   ├── ui/            # frame.ui: SSR rendering
│   ├── jobs/          # frame.jobs: enqueue, retry, schedule
│   ├── locale/        # frame.locale: translations, plurals
│   ├── mcp/           # frame.mcp: tool/resource/prompt
│   └── canvas/        # frame.canvas: drawing primitives
├── bridge/            # host bridge contract tests (Phase 4)
└── e2e/               # full app boot + HTTP roundtrip (Phase 5)
```

## Adding a test

1. Read [tests/CONVENTIONS.md](../../CONVENTIONS.md).
2. Place the file in the matching subfolder using `<feature>-test.cln` naming.
3. End the file with an explicit `PASS: <suite>` or `FAIL: <suite>` line.
4. Run `./scripts/run-framework-tests.sh runtime` locally before committing.
