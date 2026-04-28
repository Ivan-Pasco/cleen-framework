# Frame Platforms & Packaging (08)

**Project:** Frame – Full-Stack Framework for Clean Language  
**Version:** 1.0  
**Location:** `/documents/specification/08_frame_platforms.md`

---

## 1. Purpose

This document explains how to package and run a Frame app on:
- **Web** (baseline) and **PWA** (installable)
- **Mobile** (Android/iOS with Capacitor)
- **Desktop** (Windows/Linux/macOS with Tauri)
- **Server** (Node/Rust hosts) and **CLI/daemon**

We keep one codebase: **Clean → WASM**, HTML-first UI, and a small **Host Bridge** per platform.

---

## 2. Quick Targets

```bash
cleen build --target=web        # static web build
cleen build --target=pwa        # web + manifest + service worker
cleen build --target=mobile     # Android/iOS wrapper (Capacitor)
cleen build --target=desktop    # Windows/Linux/macOS wrapper (Tauri)
cleen build --target=server     # server artifacts (Node/Rust host)
cleen build --target=cli        # CLI/daemon bundle
```

Output folder: `/dist`.

---

## 3. Web (Baseline)

- **Use when:** fastest path to users.
- **Host:** any static host/CDN (Netlify, Vercel static, S3+CloudFront).
- **Artifacts:** `/public/*` + compiled WASM bundles.

### Steps
```bash
cleen build --target=web
```
Upload `/public` and `/dist` assets to your host.

---

## 4. PWA (Installable)

- **Use when:** Add to Home Screen, offline cache, basic notifications.
- **Artifacts:** `public/manifest.json`, `public/sw.js`.

### Steps
```bash
cleen build --target=pwa
```

**Minimal `public/manifest.json`**
```json
{ "name":"My App","short_name":"MyApp","start_url":"/","display":"standalone","background_color":"#ffffff","theme_color":"#2563eb","icons":[{"src":"/icons/icon-192.png","sizes":"192x192","type":"image/png"}] }
```

**Minimal `public/sw.js`**
```js
self.addEventListener('install', e=>self.skipWaiting());
self.addEventListener('activate', e=>clients.claim());
```

---

## 5. Mobile (Android & iOS with Capacitor)

- **Use when:** need camera, files, notifications, biometrics.
- **Wrapper:** Capacitor app that loads your Frame UI (HTML/WASM) and exposes native features via the **Host Bridge**.

### Steps
```bash
cleen mobile:init
cd dist/mobile/capacitor
npx cap sync
npx cap open ios    # or: npx cap open android
```

### Common plugins
- Camera, Filesystem, Push, Background Task, Biometric.

### Clean bridge (example)
```clean
bridge camera.capture
    input: { quality: integer = 80 }
    returns: { mime: string, data: bytes }
```

### Call it
```clean
bytes photo = camera.capture({ quality: 70 }).data
```

**Permissions:** add iOS `Info.plist`/Android `AndroidManifest.xml` descriptions.

---

## 6. Desktop (Windows/Linux/macOS with Tauri)

- **Use when:** want a tiny desktop app with native dialogs, FS, and auto‑update.
- **Wrapper:** Tauri (Rust host) with **Host Bridge** adapters.

### Steps
```bash
cleen desktop:init
cd dist/desktop/tauri
cargo tauri dev
```

### Filesystem bridge (sketch)
```clean
bridge fs.read  input: { path: string } returns: { data: bytes }
bridge fs.write input: { path: string, data: bytes } returns: { ok: boolean }
```

**Allowlist:** restrict Tauri APIs to only what you use.

---

## 7. Server (Node or Clean Host)

- **Use when:** deploy API/UI from a server.

### Node example (Docker)
```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY dist/ ./dist
EXPOSE 8080
CMD ["node","dist/server/index.js"]
```

### Health endpoints
- `GET /health/ready` → app compiled, DB reachable
- `GET /health/live`  → process running

**Env:** keep secrets in environment variables (`JWT_SECRET`, `DATABASE_URL`).

---

## 8. CLI / Daemon

- **Use when:** local tools, background workers, schedulers.

### Systemd (sketch)
```ini
[Service]
ExecStart=/usr/bin/node /opt/myapp/dist/server/index.js
Restart=always
```

---

## 9. Storage & Offline

- **Web/PWA:** IndexedDB for structured data; Cache Storage for assets.
- **Mobile/Desktop:** SQLite via bridge; same API shape across platforms.
- **Sync pattern:** queue writes locally → background sync → reconcile on server.

---

## 10. Security Checklist

- **PWA:** HTTPS only, strict CSP, request permissions after a user gesture.
- **Mobile:** Declare runtime permissions with user-facing descriptions.
- **Desktop:** Tauri allowlist, disable shell/FS beyond what you use.
- **Server:** Rotate secrets, enable HSTS, log without PII.

---

## 11. Host Bridge: Capability Matrix

| Capability | Web/PWA | Mobile (Capacitor) | Desktop (Tauri) | Server/CLI |
|---|---|---|---|---|
| HTTP fetch | ✅ | ✅ | ✅ | ✅ |
| Filesystem | ➖ (sandboxed) | ✅ | ✅ | ✅ |
| Camera/Media | ✅(grant) | ✅ | ➖ | ➖ |
| Notifications | ✅ | ✅ | ✅ | ➖ |
| SQLite | via WASM/OPFS | ✅ | ✅ | ✅ |
| Push (remote) | ✅ | ✅ | ➖ | ➖ |

Legend: ✅ built‑in or easy, ➖ limited/not typical.

---

## 12. CLI Helpers

```bash
cleen pwa:init
cleen mobile:init
cleen mobile:plugin camera
cleen desktop:init
cleen desktop:adapter fs
cleen server:init
```

---

## 13. AI Development Notes

- Deterministic file locations and bridge names help AI agents scaffold wrappers.
- Use normalized **bridge contracts** (see [frame_bridge_contracts.md](frame_bridge_contracts.md)).
- Prefer **JSON output** from scripts to keep logs machine‑readable.

---

## 14. File Locations

- PWA assets: `/public/manifest.json`, `/public/sw.js`
- Mobile wrapper: `/dist/mobile/capacitor/*`
- Desktop wrapper: `/dist/desktop/tauri/*`
- Server bundle: `/dist/server/*`
- CLI bundle: `/dist/cli/*`

---

**End of Document 08**

