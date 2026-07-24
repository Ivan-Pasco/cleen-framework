#!/usr/bin/env python3
"""refresh-manifest-hash.py — recompute size_bytes / sha256 in build-manifest.json.

Invoked by plugin build.sh scripts after the wasm-tools strip pass, because
`cln compile --target=plugin` writes build-manifest.json before the strip runs
and never re-visits it. Without this fixup, the manifest's integrity fields
describe an intermediate wasm that never ships.

Tracks: bug #73177fafd75f (FRAME-BUILD-manifest-drift).

Usage:
    refresh-manifest-hash.py <plugin_dir>

Rewrites <plugin_dir>/build-manifest.json in place so the artefact whose
"purpose" is "main_module" carries the current on-disk plugin.wasm's real
size and sha256. Preserves formatting, insertion order, and all other fields.
"""

import hashlib
import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: refresh-manifest-hash.py <plugin_dir>", file=sys.stderr)
        return 2

    plugin_dir = Path(sys.argv[1])
    wasm_path = plugin_dir / "plugin.wasm"
    manifest_path = plugin_dir / "build-manifest.json"

    if not wasm_path.is_file():
        print(f"error: {wasm_path} not found", file=sys.stderr)
        return 1
    if not manifest_path.is_file():
        print(f"error: {manifest_path} not found", file=sys.stderr)
        return 1

    wasm_bytes = wasm_path.read_bytes()
    size = len(wasm_bytes)
    sha = hashlib.sha256(wasm_bytes).hexdigest()

    with manifest_path.open("r", encoding="utf-8") as f:
        manifest = json.load(f)

    artefacts = manifest.get("artifacts", [])
    updated = False
    for art in artefacts:
        if art.get("purpose") == "main_module" and art.get("name") == "plugin.wasm":
            if art.get("size_bytes") != size or art.get("sha256") != sha:
                art["size_bytes"] = size
                art["sha256"] = sha
                updated = True
            break
    else:
        print(
            f"error: no main_module artefact named plugin.wasm in {manifest_path}",
            file=sys.stderr,
        )
        return 1

    if updated:
        # Match cln compile's serialisation: 2-space indent, no trailing newline.
        with manifest_path.open("w", encoding="utf-8") as f:
            json.dump(manifest, f, indent=2)
        print(f"refreshed manifest: size={size} sha256={sha[:16]}...")
    else:
        print("manifest already matches on-disk plugin.wasm; no change")

    return 0


if __name__ == "__main__":
    sys.exit(main())
